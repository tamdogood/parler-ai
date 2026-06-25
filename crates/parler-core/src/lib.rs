//! parler-core — the NATS/JetStream binding, the endpoint, and host-integration contracts.
//!
//! Port of Cotal `packages/core`. Today this crate ships the **host-integration contracts**
//! (`Runtime`/`AgentHandle`/`Terminal`/`Launch`) that the manager and the runtime extensions
//! (`parler-cmux`, `parler-tmux`) share — these are pure traits/types with no mesh dependency.
//!
//! The endpoint (the NATS client: connection, streams, presence, channels, delivery, Plane-3 — the
//! port of the 133 KB `endpoint.ts`) is the next major piece and is not yet implemented.
//!
//! Unlike Cotal's TypeScript (which uses an import-time global `Registry` for self-registration),
//! Parler wires backends by **explicit construction** (the manager matches a [`RuntimeKind`] to a
//! constructor) — same behavior, no global mutable state.

pub mod error;
pub mod launch;
pub mod runtime;
pub mod terminal;

pub use error::RuntimeError;
pub use launch::{LaunchOpts, LaunchSpec};
pub use runtime::{AgentHandle, AgentStatus, AttachSession, Runtime, RuntimeKind};
pub use terminal::{Pane, Split, SplitDirection, Tab, TerminalLayout};
