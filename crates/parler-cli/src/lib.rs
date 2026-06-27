//! parler-cli — the `parler` command-line surface.
//!
//! Every networked subcommand is a thin wrapper over [`parler_connector::MeshAgent`]: load the
//! local identity, connect to the hub, do one op, print. `parler hub` runs the bus in-process and
//! `parler mcp` exposes the same ops as MCP tools (see [`mcp`]).

pub mod mcp;

use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use parler_connector::{Config, MeshAgent};
use parler_protocol::{
    AgentSkill, DirectoryEntry, DiscoverScope, Part, RoomKind, StoredMessage, Target, Visibility,
};
use std::sync::Arc;

#[derive(Parser)]
#[command(
    name = "parler",
    version,
    about = "Parler — Slack for agents: 1:1 / many:1 / 1:many messaging + a shared memory store"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run the hub (the message bus + memory store).
    Hub(HubArgs),
    /// Create this agent's identity and point it at a hub.
    Init(InitArgs),
    /// Mint an invite code/link to hand to another agent (default: a 1:1 DM).
    Invite(InviteArgs),
    /// Redeem a pasted invite code/link.
    Join {
        /// The code (or full link) the other agent gave you.
        code: String,
    },
    /// Join a service queue as a worker (many-to-one), then `recv` it for tasks.
    Serve {
        service: String,
    },
    /// Publish this agent's discovery card to the hub directory (default: private).
    Register(RegisterArgs),
    /// Discover agents — the whole hub (default) or just the public directory (--public).
    Discover(DiscoverArgs),
    /// Show a single agent's directory card by id.
    Card {
        id: String,
    },
    /// Mint a directory token to paste into the website to view this hub's private directory.
    Token(TokenArgs),
    /// Send a message (one of --room / --to / --service).
    Send(SendArgs),
    /// Pull new messages for a room (advances your cursor unless --since/--all).
    Recv(RecvArgs),
    /// Write a fact to the shared memory store.
    Remember(RememberArgs),
    /// Recall facts by full-text query (returns only relevant rows — low token cost).
    Recall(RecallArgs),
    /// List the rooms you belong to, with unread counts.
    Rooms,
    /// Show who is in a room.
    Roster {
        #[arg(long)]
        room: String,
    },
    /// Advertise your presence status.
    Presence {
        /// One of: idle | working | waiting | offline (free-form).
        status: String,
        #[arg(long)]
        activity: Option<String>,
    },
    /// Print this agent's identity and hub.
    Whoami,
    /// Run the MCP server (stdio) exposing the parler_* tools to an MCP host.
    Mcp,
}

#[derive(Args)]
struct HubArgs {
    #[arg(long, env = "PARLER_HUB_ADDR", default_value = "127.0.0.1:7070")]
    addr: String,
    /// SQLite file for durable storage. Omit for in-memory (lost on exit).
    #[arg(long, env = "PARLER_HUB_DB")]
    db: Option<String>,
    /// Public base URL advertised in invite links. Defaults to `parler://<addr>`.
    #[arg(long, env = "PARLER_HUB_URL")]
    url: Option<String>,
    /// Display name for this hub (the workspace name shown in the directory/site).
    #[arg(long, env = "PARLER_HUB_NAME", default_value = "Parler Hub")]
    name: String,
    /// Run a public hub (world-readable directory). Omit for a private, token-gated hub.
    #[arg(long, env = "PARLER_HUB_PUBLIC")]
    public: bool,
}

#[derive(Args)]
struct InitArgs {
    /// Hub address/URL (host:port, ws://, or parler://).
    #[arg(long, default_value = "parler://127.0.0.1:7070")]
    hub: String,
    /// Display name (defaults to $USER).
    #[arg(long)]
    name: Option<String>,
    /// The role this agent plays (planner, reviewer, …).
    #[arg(long)]
    role: Option<String>,
    /// Overwrite an existing identity.
    #[arg(long)]
    force: bool,
}

#[derive(Args)]
struct InviteArgs {
    /// Create a group channel room (one-to-many) with this name.
    #[arg(long)]
    group: Option<String>,
    /// Create a service worker queue (many-to-one) with this name.
    #[arg(long)]
    service: Option<String>,
    /// Invite lifetime in seconds (default 86400).
    #[arg(long)]
    ttl: Option<u64>,
    /// How many agents may redeem it (channel/service only; a DM is always single-use).
    #[arg(long)]
    max_uses: Option<u32>,
}

#[derive(Args)]
struct SendArgs {
    /// Send to a channel room (one-to-many).
    #[arg(long)]
    room: Option<String>,
    /// Send a DM to a peer agent id (one-to-one).
    #[arg(long)]
    to: Option<String>,
    /// Send to a service queue (many-to-one).
    #[arg(long)]
    service: Option<String>,
    /// The message text.
    #[arg(required = true, trailing_var_arg = true)]
    text: Vec<String>,
}

#[derive(Args)]
struct RecvArgs {
    #[arg(long)]
    room: String,
    /// Pull messages with seq greater than this (does not advance your cursor).
    #[arg(long)]
    since: Option<i64>,
    /// Re-read the full history (equivalent to --since 0).
    #[arg(long)]
    all: bool,
    #[arg(long)]
    limit: Option<u32>,
}

#[derive(Args)]
struct RememberArgs {
    /// A stable key — re-remembering the same key overwrites (idempotent).
    #[arg(long)]
    key: Option<String>,
    /// Scope the fact to a room (default: your private memory).
    #[arg(long)]
    room: Option<String>,
    #[arg(required = true, trailing_var_arg = true)]
    text: Vec<String>,
}

#[derive(Args)]
struct RecallArgs {
    /// Limit the search to a room (default: all your reachable memory).
    #[arg(long)]
    room: Option<String>,
    #[arg(long)]
    limit: Option<u32>,
    #[arg(required = true, trailing_var_arg = true)]
    query: Vec<String>,
}

#[derive(Args)]
struct RegisterArgs {
    /// Make this agent discoverable by anyone (public directory). Default: private (same-hub only).
    #[arg(long)]
    public: bool,
    /// A capability tag (repeatable): --tag planning --tag ops.
    #[arg(long = "tag")]
    tags: Vec<String>,
    /// A skill id (repeatable): --skill code-review.
    #[arg(long = "skill")]
    skills: Vec<String>,
    /// A short description of what this agent does.
    #[arg(long)]
    describe: Option<String>,
}

#[derive(Args)]
struct DiscoverArgs {
    /// Search only the public directory (default: the whole hub).
    #[arg(long)]
    public: bool,
    /// Filter by a capability tag.
    #[arg(long)]
    tag: Option<String>,
    /// Filter by a skill.
    #[arg(long)]
    skill: Option<String>,
    /// Filter by presence status (idle/working/waiting/offline).
    #[arg(long)]
    status: Option<String>,
    #[arg(long)]
    limit: Option<u32>,
    /// Free-text query over name / tags / skills.
    #[arg(trailing_var_arg = true)]
    query: Vec<String>,
}

#[derive(Args)]
struct TokenArgs {
    /// Token lifetime in seconds (default 3600).
    #[arg(long)]
    ttl: Option<u64>,
}

/// Entry point for the `parler` binary.
pub async fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Hub(a) => cmd_hub(a).await,
        Cmd::Init(a) => cmd_init(a),
        Cmd::Invite(a) => cmd_invite(a).await,
        Cmd::Join { code } => cmd_join(code).await,
        Cmd::Serve { service } => cmd_serve(service).await,
        Cmd::Register(a) => cmd_register(a).await,
        Cmd::Discover(a) => cmd_discover(a).await,
        Cmd::Card { id } => cmd_card(id).await,
        Cmd::Token(a) => cmd_token(a).await,
        Cmd::Send(a) => cmd_send(a).await,
        Cmd::Recv(a) => cmd_recv(a).await,
        Cmd::Remember(a) => cmd_remember(a).await,
        Cmd::Recall(a) => cmd_recall(a).await,
        Cmd::Rooms => cmd_rooms().await,
        Cmd::Roster { room } => cmd_roster(room).await,
        Cmd::Presence { status, activity } => cmd_presence(status, activity).await,
        Cmd::Whoami => cmd_whoami(),
        Cmd::Mcp => mcp::serve_stdio().await,
    }
}

async fn connect() -> Result<MeshAgent> {
    let cfg = Config::load()?;
    MeshAgent::connect(&cfg).await
}

async fn cmd_hub(a: HubArgs) -> Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()))
        .try_init();
    let store = parler_hub::Store::open(a.db.as_deref().map(std::path::Path::new))?;
    let public_url = a.url.unwrap_or_else(|| format!("parler://{}", a.addr));
    let mode = if a.public { parler_hub::HubMode::Public } else { parler_hub::HubMode::Private };
    let state = Arc::new(parler_hub::HubState { store, public_url, name: a.name, mode });
    let listener = tokio::net::TcpListener::bind(&a.addr).await?;
    let actual = listener.local_addr()?;
    println!(
        "parler-hub up · ws://{actual}/ws · {} hub '{}' · db: {}",
        state.mode.as_str(),
        state.name,
        a.db.as_deref().unwrap_or(":memory:")
    );
    parler_hub::serve(listener, state).await
}

fn cmd_init(a: InitArgs) -> Result<()> {
    if Config::exists() && !a.force {
        bail!("already initialized — pass --force to overwrite the existing identity");
    }
    let name = a
        .name
        .unwrap_or_else(|| std::env::var("USER").unwrap_or_else(|_| "agent".into()));
    let cfg = Config::create(a.hub, name, a.role)?;
    cfg.save()?;
    println!("✓ identity created");
    println!("  id:   {}", cfg.identity.id);
    println!(
        "  name: {}{}",
        cfg.name,
        cfg.role.as_deref().map(|r| format!(" ({r})")).unwrap_or_default()
    );
    println!("  hub:  {}", cfg.hub_url);
    println!("  saved to {}/config.json", parler_connector::home_dir().display());
    Ok(())
}

async fn cmd_invite(a: InviteArgs) -> Result<()> {
    if a.group.is_some() && a.service.is_some() {
        bail!("--group and --service are mutually exclusive");
    }
    let (kind, room) = if let Some(g) = a.group {
        (RoomKind::Channel, Some(g))
    } else if let Some(s) = a.service {
        (RoomKind::Service, Some(s))
    } else {
        (RoomKind::Dm, None)
    };
    let mut ag = connect().await?;
    let inv = ag.invite(kind, room, a.ttl, a.max_uses).await?;
    println!("✓ invite ready — {} room '{}'", inv.kind.as_str(), inv.room);
    println!();
    println!("    code: {}", inv.code);
    println!("    link: {}", inv.url);
    println!();
    println!("Hand it to another agent and have it run:  parler join {}", inv.code);
    Ok(())
}

async fn cmd_join(code: String) -> Result<()> {
    let mut ag = connect().await?;
    let (room, kind) = ag.join(&code).await?;
    println!("✓ joined {} room '{}'", kind.as_str(), room);
    println!("  receive with:  parler recv --room {room}");
    Ok(())
}

async fn cmd_serve(service: String) -> Result<()> {
    let mut ag = connect().await?;
    let room = ag.serve(&service).await?;
    println!("✓ serving '{service}' (room '{room}')");
    println!("  receive tasks with:  parler recv --room {room}");
    Ok(())
}

async fn cmd_register(a: RegisterArgs) -> Result<()> {
    let visibility = if a.public { Visibility::Public } else { Visibility::Private };
    let skills = a
        .skills
        .into_iter()
        .map(|s| AgentSkill { id: s.clone(), name: s, description: None })
        .collect();
    let mut ag = connect().await?;
    let (visibility, verified) = ag.register(visibility, a.tags, skills, a.describe).await?;
    let sig = if verified { "signature verified ✓" } else { "unsigned" };
    println!("✓ registered in the directory as {} ({sig})", visibility.as_str());
    println!("  discover with:  parler discover{}", if visibility == Visibility::Public { " --public" } else { "" });
    Ok(())
}

async fn cmd_discover(a: DiscoverArgs) -> Result<()> {
    let scope = if a.public { DiscoverScope::Public } else { DiscoverScope::Hub };
    let query = (!a.query.is_empty()).then(|| a.query.join(" "));
    let mut ag = connect().await?;
    let agents = ag.discover(scope, query, a.tag, a.skill, a.status, a.limit).await?;
    if agents.is_empty() {
        println!("(no agents found)");
        return Ok(());
    }
    let scope_label = if a.public { "public directory" } else { "hub" };
    println!("{} agent(s) in the {scope_label}:", agents.len());
    for e in &agents {
        println!("{}", render_entry(e));
    }
    Ok(())
}

async fn cmd_card(id: String) -> Result<()> {
    let mut ag = connect().await?;
    match ag.lookup(&id).await? {
        Some(e) => print!("{}", render_entry_full(&e)),
        None => println!("(no directory card for '{id}')"),
    }
    Ok(())
}

async fn cmd_token(a: TokenArgs) -> Result<()> {
    let mut ag = connect().await?;
    let (token, expires_at) = ag.mint_directory_token(a.ttl).await?;
    println!("✓ directory token (expires at {expires_at}):");
    println!();
    println!("    {token}");
    println!();
    println!("Paste it into the website's \"hub view\" to browse this hub's private directory.");
    Ok(())
}

async fn cmd_send(a: SendArgs) -> Result<()> {
    let target = match (a.room, a.to, a.service) {
        (Some(r), None, None) => Target::Room { room: r },
        (None, Some(t), None) => Target::Dm { agent: t },
        (None, None, Some(s)) => Target::Service { service: s },
        (None, None, None) => bail!("specify a destination: --room, --to, or --service"),
        _ => bail!("specify exactly one of --room, --to, --service"),
    };
    let text = a.text.join(" ");
    let mut ag = connect().await?;
    let (_id, seq, room) = ag.send_text(target, &text).await?;
    println!("✓ sent to '{room}' (seq {seq})");
    Ok(())
}

async fn cmd_recv(a: RecvArgs) -> Result<()> {
    let since = if a.all { Some(0) } else { a.since };
    let mut ag = connect().await?;
    let (msgs, cursor) = ag.pull(&a.room, since, a.limit).await?;
    if msgs.is_empty() {
        println!("(no new messages in '{}')", a.room);
        return Ok(());
    }
    for m in &msgs {
        println!("{}", render_message(m));
    }
    println!("— cursor at {cursor} —");
    Ok(())
}

async fn cmd_remember(a: RememberArgs) -> Result<()> {
    let text = a.text.join(" ");
    let mut ag = connect().await?;
    ag.remember(&text, a.key, a.room).await?;
    println!("✓ remembered");
    Ok(())
}

async fn cmd_recall(a: RecallArgs) -> Result<()> {
    let query = a.query.join(" ");
    let mut ag = connect().await?;
    let hits = ag.recall(&query, a.room, a.limit).await?;
    if hits.is_empty() {
        println!("(nothing recalled for '{query}')");
        return Ok(());
    }
    for h in &hits {
        let scope = h.room.as_deref().map(|r| format!("#{r}")).unwrap_or_else(|| "private".into());
        let key = h.key.as_deref().map(|k| format!("[{k}] ")).unwrap_or_default();
        println!("• {key}{} ({scope})", h.text);
    }
    Ok(())
}

async fn cmd_rooms() -> Result<()> {
    let mut ag = connect().await?;
    let rooms = ag.rooms().await?;
    if rooms.is_empty() {
        println!("(no rooms yet — `parler invite` or `parler join`)");
        return Ok(());
    }
    for r in &rooms {
        let unread = if r.unread > 0 { format!("  ({} unread)", r.unread) } else { String::new() };
        println!("#{}  [{}]  {} member(s){unread}", r.name, r.kind.as_str(), r.members);
    }
    Ok(())
}

async fn cmd_roster(room: String) -> Result<()> {
    let mut ag = connect().await?;
    let entries = ag.roster(&room).await?;
    println!("members of '{room}':");
    for e in &entries {
        let role = e.role.as_deref().map(|r| format!(" ({r})")).unwrap_or_default();
        let act = e.activity.as_deref().map(|a| format!(" — {a}")).unwrap_or_default();
        println!("  {} {}{role}  [{}]{act}", e.name, e.id, e.status);
    }
    Ok(())
}

async fn cmd_presence(status: String, activity: Option<String>) -> Result<()> {
    let mut ag = connect().await?;
    ag.presence(&status, activity).await?;
    println!("✓ presence: {status}");
    Ok(())
}

fn cmd_whoami() -> Result<()> {
    let cfg = Config::load()?;
    println!("id:   {}", cfg.identity.id);
    println!(
        "name: {}{}",
        cfg.name,
        cfg.role.as_deref().map(|r| format!(" ({r})")).unwrap_or_default()
    );
    println!("hub:  {}", cfg.hub_url);
    Ok(())
}

/// One-line directory entry: `● name (role)  Uid…  [public ✓]  working  #tag …`.
fn render_entry(e: &DirectoryEntry) -> String {
    let role = e.card.role.as_deref().map(|r| format!(" ({r})")).unwrap_or_default();
    let vis = if e.verified {
        format!("{} ✓", e.visibility.as_str())
    } else {
        e.visibility.as_str().to_string()
    };
    let tags = e
        .card
        .tags
        .as_deref()
        .map(|t| t.iter().map(|x| format!("#{x}")).collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    format!(
        "● {}{role}  {}  [{}]  {}  {}",
        e.card.name, e.card.id, vis, e.status, tags
    )
}

/// Multi-line directory card for `parler card <id>`.
fn render_entry_full(e: &DirectoryEntry) -> String {
    let mut out = String::new();
    out.push_str(&format!("name:    {}\n", e.card.name));
    out.push_str(&format!("id:      {}\n", e.card.id));
    if let Some(role) = &e.card.role {
        out.push_str(&format!("role:    {role}\n"));
    }
    out.push_str(&format!("hub:     {}\n", e.hub));
    out.push_str(&format!(
        "visible: {} ({})\n",
        e.visibility.as_str(),
        if e.verified { "signature verified ✓" } else { "unverified" }
    ));
    out.push_str(&format!("status:  {}\n", e.status));
    if let Some(d) = &e.card.description {
        out.push_str(&format!("about:   {d}\n"));
    }
    if let Some(tags) = &e.card.tags {
        out.push_str(&format!("tags:    {}\n", tags.join(", ")));
    }
    if let Some(skills) = &e.card.skills {
        let s = skills.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join(", ");
        out.push_str(&format!("skills:  {s}\n"));
    }
    out
}

/// Render the text of a message's parts (text joined; data/extension parts noted).
pub fn render_parts(parts: &[Part]) -> String {
    let mut out = Vec::new();
    for p in parts {
        match p {
            Part::Text(t) => out.push(t.clone()),
            Part::Data(d) => out.push(format!("[data] {d}")),
            Part::Extension { kind, .. } => out.push(format!("[{kind}]")),
        }
    }
    out.join(" ")
}

/// One line: `[seq] name (role): text`.
pub fn render_message(m: &StoredMessage) -> String {
    let who = m
        .from
        .role
        .as_deref()
        .map(|r| format!("{} ({r})", m.from.name))
        .unwrap_or_else(|| m.from.name.clone());
    format!("[{}] {}: {}", m.seq, who, render_parts(&m.parts))
}
