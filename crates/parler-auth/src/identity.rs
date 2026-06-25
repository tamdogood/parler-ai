//! A locally-generated agent identity (an nkey user keypair). Port of Cotal `identity.ts`.
//!
//! The public key is the **stable id** used identically everywhere — `card.id`, the subject-encoded
//! sender token, the JWT subject, and the DM durable name. The seed is the private half; it never
//! goes on the wire and is folded into a creds file the endpoint loads to authenticate as this id.

use crate::error::AuthError;
use data_encoding::BASE64URL_NOPAD;
use nkeys::KeyPair;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// User nkey public key (`U…`). The stable agent id.
    pub id: String,
    /// User nkey seed (`SU…`). Private — kept off the wire.
    pub seed: String,
}

/// Generate a fresh user nkey identity locally.
pub fn new_identity() -> Result<Identity, AuthError> {
    let kp = KeyPair::new_user();
    let seed = kp.seed().map_err(|e| AuthError::Nkeys(e.to_string()))?;
    Ok(Identity {
        id: kp.public_key(),
        seed,
    })
}

/// The stable id carried by a creds file: the agent's nkey public key, derived from the seed block
/// and cross-checked against the JWT subject (a mismatch ⇒ a corrupt/spliced creds file).
pub fn id_from_creds(creds: &str) -> Result<String, AuthError> {
    let seed = extract_block(creds, "USER NKEY SEED")
        .ok_or_else(|| AuthError::Creds("no user nkey seed block found".into()))?;
    let kp = KeyPair::from_seed(seed.trim()).map_err(|e| AuthError::Creds(format!("bad seed: {e}")))?;
    let id = kp.public_key();
    if let Some(jwt) = extract_block(creds, "NATS USER JWT") {
        if let Some(sub) = jwt_subject(jwt.trim()) {
            if sub != id {
                return Err(AuthError::Creds(format!(
                    "seed identity {id} != JWT subject {sub}"
                )));
            }
        }
    }
    Ok(id)
}

/// Extract the content between `-----BEGIN <label>-----` and `------END <label>------` (tolerant).
fn extract_block(creds: &str, label: &str) -> Option<String> {
    let begin = format!("BEGIN {label}");
    let end = format!("END {label}");
    let b = creds.find(&begin)?;
    let after_begin = creds[b..].find('\n')? + b + 1;
    let e_marker = creds[after_begin..].find(&end)? + after_begin;
    // Back up to the start of the line carrying the END marker so its leading dashes are excluded.
    let line_start = creds[..e_marker].rfind('\n').map(|n| n + 1).unwrap_or(after_begin);
    Some(creds[after_begin..line_start].trim().to_string())
}

/// Read the `sub` claim out of a NATS JWT (the middle base64url-nopad segment).
fn jwt_subject(jwt: &str) -> Option<String> {
    let payload = jwt.split('.').nth(1)?;
    let bytes = BASE64URL_NOPAD.decode(payload.as_bytes()).ok()?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    v.get("sub")?.as_str().map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_identity_is_a_user_key() {
        let id = new_identity().unwrap();
        assert!(id.id.starts_with('U'));
        assert!(id.seed.starts_with("SU"));
        // The seed re-derives the same public key.
        let kp = KeyPair::from_seed(&id.seed).unwrap();
        assert_eq!(kp.public_key(), id.id);
    }
}
