//! Live de-risking test for the top technical risk: the decentralized NATS JWT chain.
//!
//! Boots a real `nats-server` with a parler-minted operator/account/system JWT trust chain, then
//! connects with parler-minted user creds and asserts the broker enforces the agent profile ACL:
//! the account has JetStream (manager creates a stream), and an agent's publish to a DECLARED
//! channel is delivered while a publish to an UNDECLARED channel is rejected by the server.

use futures::StreamExt;
use parler_auth::{
    create_space_auth, mint_creds, new_identity, server_config, MintOpts, Profile, ServerConfigOpts,
};
use parler_protocol::{chat_stream, chat_subject, space_prefix};
use std::time::{Duration, Instant};

struct ServerGuard(std::process::Child);
impl Drop for ServerGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn nats_server_bin() -> String {
    std::env::var("PARLER_NATS_SERVER").unwrap_or_else(|_| {
        format!("{}/../../.context/bin/nats-server", env!("CARGO_MANIFEST_DIR"))
    })
}

async fn wait_ready(port: u16) {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
            return;
        }
        assert!(Instant::now() < deadline, "nats-server not ready on :{port}");
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn connect(creds: &str, url: &str, inbox: Option<String>) -> async_nats::Client {
    let mut opts = async_nats::ConnectOptions::with_credentials(creds).expect("parse creds");
    if let Some(p) = inbox {
        opts = opts.custom_inbox_prefix(p);
    }
    tokio::time::timeout(Duration::from_secs(8), opts.connect(url))
        .await
        .expect("connect timed out (auth rejected?)")
        .expect("connect failed")
}

#[tokio::test]
async fn agent_acl_enforced_against_live_server() {
    let bin = nats_server_bin();
    assert!(
        std::path::Path::new(&bin).exists(),
        "nats-server not found at {bin} (set PARLER_NATS_SERVER)"
    );
    let space = "main";
    let auth = create_space_auth(space).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let store = dir.path().join("js");
    std::fs::create_dir_all(&store).unwrap();
    let port = free_port();
    let cfg = server_config(
        &auth,
        &ServerConfigOpts {
            port,
            host: "127.0.0.1".into(),
            store_dir: store.to_string_lossy().into_owned(),
        },
    );
    let cfg_path = dir.path().join("server.conf");
    std::fs::write(&cfg_path, cfg).unwrap();

    let child = std::process::Command::new(&bin)
        .arg("-c")
        .arg(&cfg_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("spawn nats-server");
    let _guard = ServerGuard(child);
    wait_ready(port).await;
    let url = format!("nats://127.0.0.1:{port}");

    // Manager (allow-all): create the CHAT stream (proves the account has JetStream) + tap chat.>.
    let mgr_id = new_identity().unwrap();
    let mgr_creds = mint_creds(&auth, &mgr_id, Profile::Manager, &MintOpts::default()).unwrap();
    let mgr = connect(&mgr_creds, &url, None).await;
    let js = async_nats::jetstream::new(mgr.clone());
    js.create_stream(async_nats::jetstream::stream::Config {
        name: chat_stream(space),
        subjects: vec![format!("{}.chat.>", space_prefix(space))],
        ..Default::default()
    })
    .await
    .expect("manager creates CHAT stream (account JetStream enabled)");
    let mut sub = mgr
        .subscribe(format!("{}.chat.>", space_prefix(space)))
        .await
        .expect("manager subscribe");
    mgr.flush().await.unwrap();

    // Agent: declares only #general. Publish there (allowed) + #secret (denied) + a sentinel.
    let agent_id = new_identity().unwrap();
    let agent_creds = mint_creds(
        &auth,
        &agent_id,
        Profile::Agent,
        &MintOpts {
            allow_publish: vec!["general".into()],
            allow_subscribe: vec!["general".into()],
            ..Default::default()
        },
    )
    .unwrap();
    let agent = connect(&agent_creds, &url, Some(format!("_INBOX_{}", agent_id.id))).await;

    let allowed = chat_subject(space, &agent_id.id, "general").unwrap();
    let denied = chat_subject(space, &agent_id.id, "secret").unwrap();
    agent.publish(allowed.clone(), "ok".into()).await.unwrap();
    agent.publish(denied.clone(), "nope".into()).await.unwrap();
    agent.publish(allowed.clone(), "sentinel".into()).await.unwrap();
    agent.flush().await.unwrap();

    let mut seen: Vec<String> = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_millis(300), sub.next()).await {
            Ok(Some(m)) => {
                let body = String::from_utf8_lossy(&m.payload).to_string();
                let stop = body == "sentinel";
                seen.push(body);
                if stop {
                    break;
                }
            }
            Ok(None) => break,
            Err(_) => {}
        }
    }

    assert!(seen.contains(&"ok".to_string()), "allowed publish missing: {seen:?}");
    assert!(seen.contains(&"sentinel".to_string()), "sentinel missing: {seen:?}");
    assert!(
        !seen.contains(&"nope".to_string()),
        "DENIED publish leaked through the broker ACL: {seen:?}"
    );
}
