use thiserror::Error;

/// Errors from a runtime backend (spawning/attaching/driving agents).
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("{0} runtime is not reachable")]
    Unavailable(String),
    #[error("{kind} runtime: unsafe agent name {name:?} (allowed: letters, digits, _ . -)")]
    UnsafeName { kind: String, name: String },
    #[error("{0}")]
    Cli(String),
    #[error("not supported: {0}")]
    NotSupported(String),
    #[error("io: {0}")]
    Io(String),
}
