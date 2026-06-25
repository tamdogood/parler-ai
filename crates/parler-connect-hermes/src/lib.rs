//! parler-connect-hermes — the Hermes (Nous Research) connector.
//!
//! Port of Cotal `extensions/connector-hermes`. Hermes is a long-lived gateway daemon that creates a
//! fresh agent per inbound message, so the mesh endpoint must outlive every turn — it can't ride
//! inside a per-turn server. The architecture is therefore split across two languages:
//!
//! - a **Python plugin** (`plugin/parler/`, loaded by Hermes) — the gateway platform adapter,
//!   lifecycle→presence hooks, and the `parler_*` tools. It stays Python because Hermes loads it.
//! - this **Rust sidecar** — owns the single mesh agent and exposes the [`bridge`] socket the plugin
//!   talks to (inbound push / outbound replies / tool calls).
//!
//! What's implemented here: the bridge **wire protocol** ([`bridge`]) + its serial ack-on-surface
//! **state machine** (fully tested), the unix-socket **server** ([`serve`]) over a [`MeshHandle`]
//! seam, and the **launch recipe** ([`launch`]). The live mesh plugs into [`MeshHandle`] once
//! `parler-connector`'s `MeshAgent` lands.

pub mod bridge;
pub mod launch;
pub mod serve;

pub use bridge::{
    BridgeState, InFrame, InboxItem, InboxView, OutFrame, ReplyTarget, ToolResult, WireItem,
};
pub use launch::{
    bridge_socket_path, build_launch, render_config, render_soul, setup_profile, tok, HERMES_PIN,
};
pub use serve::{serve, MeshHandle};
