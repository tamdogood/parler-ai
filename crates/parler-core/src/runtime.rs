//! Runtime backend contract. Port of Cotal `runtime.ts`.
//!
//! A [`Runtime`] is a pluggable agent backend: `pty` owns a real pseudo-terminal; `tmux`/`cmux`
//! drive a multiplexer. The manager owns an [`AgentHandle`] to *control* the process (the mesh
//! observes its presence separately).

use crate::error::RuntimeError;
use crate::launch::LaunchSpec;

/// Which backend a manager spawns through (`pty`, `tmux`, `cmux`, …). Open-ended.
pub type RuntimeKind = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus {
    Running,
    Exited,
}

/// A live attach onto a running agent's terminal — what `parler attach` consumes. PTY frames flow
/// here directly, never over the mesh. (Backends like cmux/tmux don't stream — they throw on attach.)
pub trait AttachSession: Send {
    fn cols(&self) -> u16;
    fn rows(&self) -> u16;
    /// Scrollback so a late attach sees output that already scrolled past.
    fn backlog(&self) -> Vec<u8>;
    /// Forward keystrokes to the process.
    fn write(&self, data: &str) -> Result<(), RuntimeError>;
    /// Resize the pseudo-terminal.
    fn resize(&self, cols: u16, rows: u16) -> Result<(), RuntimeError>;
}

/// An OS handle on one spawned agent — the manager owns this to control the process.
pub trait AgentHandle: Send {
    fn name(&self) -> &str;
    fn kind(&self) -> &str;
    /// OS pid when the backend owns a real process (pty/host); absent for tmux/cmux (attach-only).
    fn pid(&self) -> Option<u32> {
        None
    }
    fn status(&self) -> AgentStatus;
    /// Tear the agent down. `graceful` signals a clean exit (so the session leaves the mesh on its
    /// own) before ensuring the process/tab is gone; otherwise a hard, immediate kill.
    fn stop(&self, graceful: bool) -> Result<(), RuntimeError>;
    fn interrupt(&self) -> Result<(), RuntimeError>;
    /// Open a live attach. Errors on backends that can't stream (tmux/cmux).
    fn attach(&self) -> Result<Box<dyn AttachSession>, RuntimeError>;
}

/// A pluggable agent backend.
pub trait Runtime {
    fn kind(&self) -> &str;
    fn spawn(
        &self,
        name: &str,
        spec: &LaunchSpec,
        cwd: &str,
    ) -> Result<Box<dyn AgentHandle>, RuntimeError>;
}
