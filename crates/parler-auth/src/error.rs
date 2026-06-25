use thiserror::Error;

/// Errors from identity, JWT issuance, and provisioning.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("nkeys: {0}")]
    Nkeys(String),
    #[error(transparent)]
    Subject(#[from] parler_protocol::SubjectError),
    #[error("jwt: {0}")]
    Jwt(String),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("creds: {0}")]
    Creds(String),
    #[error("io: {0}")]
    Io(String),
}
