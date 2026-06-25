//! parler-cmux — the cmux integration: a driver over the cmux CLI plus a cmux [`CmuxRuntime`] and
//! [`CmuxTerminal`] (a [`parler_core::TerminalLayout`]).
//!
//! Port of Cotal `extensions/cmux`. The driver stays mesh-free; the manager constructs
//! [`CmuxRuntime`] when `--runtime cmux` is selected, and `parler setup` uses [`CmuxTerminal`] to
//! open/close tabs.

pub mod driver;
pub mod runtime;

pub use driver::{available, Target};
pub use runtime::{CmuxRuntime, CmuxTerminal};
