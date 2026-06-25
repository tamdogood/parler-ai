//! The sidecar's bridge server — the IO glue around [`BridgeState`]. Port of the socket half of
//! Cotal `bridge.ts` + `sidecar.ts`.
//!
//! The live mesh plugs in through the [`MeshHandle`] trait seam: today `parler-connector`'s
//! `MeshAgent` is not built, so the real implementation lands later. The ordering logic it drives is
//! fully tested in [`crate::bridge`]; this module is the unix-socket transport that calls it.

use crate::bridge::{BridgeState, InFrame, InboxItem, InboxView, OutFrame, ReplyTarget, ToolResult};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

/// The mesh operations the bridge needs — the seam over `parler-connector`'s `MeshAgent`.
#[async_trait]
pub trait MeshHandle: Send + Sync {
    /// Snapshot of the front of the buffered inbox (cheap, non-consuming).
    fn peek_front(&self) -> Option<InboxItem>;
    /// Ack + drop the front of the inbox (the agent's `drainInbox(1)`).
    async fn drain_one(&self);
    /// Broadcast a turn's reply to a channel.
    async fn send(&self, text: &str, channel: Option<&str>);
    /// DM a turn's reply to a peer (instance id or name).
    async fn dm(&self, peer: &str, text: &str);
    /// Run a `parler_*` tool by name. `Err` is a transport/unknown-tool failure (vs an in-tool error,
    /// which comes back as a [`ToolResult`] with `is_error`).
    async fn run_tool(&self, name: &str, args: serde_json::Value) -> Result<ToolResult, String>;
    /// Resolves when inbox activity may have occurred (a new incoming, or a wake) — drives proactive
    /// pumping so a buffered message reaches an idle gateway without waiting on a frame.
    async fn activity(&self);
}

/// Adapt a [`MeshHandle`] into the [`InboxView`] that [`BridgeState`] consumes.
struct HandleInbox<'a>(&'a dyn MeshHandle);
impl InboxView for HandleInbox<'_> {
    fn peek_front(&self) -> Option<InboxItem> {
        self.0.peek_front()
    }
}

/// Bind the bridge socket and serve adapter connections (one active adapter in practice — a fresh
/// connection's `subscribe` supersedes the previous). Clears a stale socket from a dead predecessor.
pub async fn serve(socket_path: &Path, handle: Arc<dyn MeshHandle>) -> std::io::Result<()> {
    let _ = std::fs::remove_file(socket_path);
    let listener = UnixListener::bind(socket_path)?;
    loop {
        let (stream, _) = listener.accept().await?;
        let h = handle.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_conn(stream, h).await {
                eprintln!("[parler-hermes/bridge] connection ended: {e}");
            }
        });
    }
}

async fn handle_conn(stream: UnixStream, handle: Arc<dyn MeshHandle>) -> std::io::Result<()> {
    let (read_half, mut write_half) = stream.into_split();
    // into_split gives us an owned read half; re-pair the writer by reconnecting is unnecessary —
    // we write through a second handle to the same stream via the OwnedWriteHalf.
    let mut lines = BufReader::new(read_half).lines();
    let mut writer = WriteHalf(&mut write_half);
    let mut state = BridgeState::new();

    loop {
        tokio::select! {
            line = lines.next_line() => {
                let Some(line) = line? else { break };
                if line.trim().is_empty() { continue; }
                let Ok(frame) = serde_json::from_str::<InFrame>(&line) else {
                    eprintln!("[parler-hermes/bridge] malformed frame dropped");
                    continue;
                };
                dispatch(&mut state, frame, handle.as_ref(), &mut writer).await?;
            }
            _ = handle.activity() => {
                let pushed = state.pump(&HandleInbox(handle.as_ref()));
                if let Some(msg) = pushed {
                    writer.send(&OutFrame::Incoming { msg }).await?;
                }
            }
        }
    }
    Ok(())
}

/// Thin owned-write-half wrapper so `dispatch` can write frames.
struct WriteHalf<'a>(&'a mut tokio::net::unix::OwnedWriteHalf);
impl WriteHalf<'_> {
    async fn send(&mut self, frame: &OutFrame) -> std::io::Result<()> {
        let mut line = serde_json::to_string(frame).unwrap_or_default();
        line.push('\n');
        self.0.write_all(line.as_bytes()).await?;
        self.0.flush().await
    }
}

async fn dispatch(
    state: &mut BridgeState,
    frame: InFrame,
    handle: &dyn MeshHandle,
    writer: &mut WriteHalf<'_>,
) -> std::io::Result<()> {
    match frame {
        InFrame::Subscribe => {
            if let Some(msg) = state.on_subscribe(&HandleInbox(handle)) {
                writer.send(&OutFrame::Incoming { msg }).await?;
            }
        }
        InFrame::Delivered { id } => {
            if state.on_delivered(&id, &HandleInbox(handle)) {
                handle.drain_one().await;
            }
            if let Some(msg) = state.pump(&HandleInbox(handle)) {
                writer.send(&OutFrame::Incoming { msg }).await?;
            }
        }
        InFrame::Reply { target, text } => {
            if !text.trim().is_empty() {
                route_reply(handle, &target, &text).await;
            }
        }
        InFrame::Tool { id, name, args } => {
            // parler_inbox is forced read-only (peek) so a tool call never races per-turn ack.
            let args = if name == "parler_inbox" {
                serde_json::json!({ "peek": true })
            } else {
                args
            };
            let out = match handle.run_tool(&name, args).await {
                Ok(r) => OutFrame::ToolResult {
                    id,
                    ok: true,
                    text: Some(r.text),
                    is_error: Some(r.is_error),
                    error: None,
                },
                Err(e) => OutFrame::ToolResult {
                    id,
                    ok: false,
                    text: None,
                    is_error: None,
                    error: Some(e),
                },
            };
            writer.send(&out).await?;
        }
    }
    Ok(())
}

async fn route_reply(handle: &dyn MeshHandle, target: &ReplyTarget, text: &str) {
    if let Some(channel) = &target.channel {
        handle.send(text, Some(channel)).await;
    } else if let Some(peer) = &target.peer_id {
        handle.dm(peer, text).await;
    }
}
