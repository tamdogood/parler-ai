//! Decentralized NATS JWT v2 issuance (operator / account / user). Port of the parts of
//! `@nats-io/jwt` that Cotal's `provision.ts` uses, hand-rolled because no Rust crate covers
//! operator tokens, `system_account`, or JetStream account limits.
//!
//! Wire algorithm (verified against the `nats-jwt` crate + nats-server): header
//! `{"typ":"JWT","alg":"ed25519-nkey"}`, claims JSON with a base32hex-nopad SHA-256 `jti`, and an
//! ed25519 signature (by the issuer nkey) over `base64url(header) + "." + base64url(claims)`.

use crate::error::AuthError;
use data_encoding::{BASE32HEX_NOPAD, BASE64URL_NOPAD};
use nkeys::KeyPair;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

const HEADER: &str = r#"{"typ":"JWT","alg":"ed25519-nkey"}"#;

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Build, hash, and sign a claims set into a NATS JWT. `signer` is the issuer keypair (must hold a
/// seed): operator self-signs its own JWT and signs accounts; the account signing key signs users.
fn encode(name: &str, sub: &str, nats: Value, signer: &KeyPair) -> Result<String, AuthError> {
    let mut claims = json!({
        "jti": "",
        "iat": now_secs(),
        "iss": signer.public_key(),
        "name": name,
        "sub": sub,
        "nats": nats,
    });
    // jti = base32hex(sha256(claims-with-empty-jti)); the server does not recompute it.
    let body0 = serde_json::to_vec(&claims)?;
    let hash = Sha256::digest(&body0);
    claims["jti"] = Value::String(BASE32HEX_NOPAD.encode(&hash));

    let body = serde_json::to_string(&claims)?;
    let header_b64 = BASE64URL_NOPAD.encode(HEADER.as_bytes());
    let body_b64 = BASE64URL_NOPAD.encode(body.as_bytes());
    let signing_input = format!("{header_b64}.{body_b64}");
    let sig = signer
        .sign(signing_input.as_bytes())
        .map_err(|e| AuthError::Jwt(format!("sign: {e}")))?;
    Ok(format!("{signing_input}.{}", BASE64URL_NOPAD.encode(&sig)))
}

/// Operator JWT (self-signed): names the system account.
pub fn encode_operator(
    name: &str,
    operator_kp: &KeyPair,
    system_account: &str,
) -> Result<String, AuthError> {
    let nats = json!({ "type": "operator", "version": 2, "system_account": system_account });
    encode(name, &operator_kp.public_key(), nats, operator_kp)
}

/// Account JWT (signed by the operator). `limits` is the flattened NATS/account/JetStream limits.
pub fn encode_account(
    name: &str,
    account_pub: &str,
    operator_kp: &KeyPair,
    signing_keys: &[String],
    limits: Value,
) -> Result<String, AuthError> {
    let nats = json!({
        "type": "account",
        "version": 2,
        "signing_keys": signing_keys,
        "limits": limits,
    });
    encode(name, account_pub, nats, operator_kp)
}

/// User JWT (signed by the account signing key). `permissions` is `{pub:{allow,deny}, sub:{allow}}`
/// (or `{}` for allow-all). `issuer_account` is the account identity key (required when signed by a
/// signing key, so the server maps the user to the account).
pub fn encode_user(
    name: &str,
    user_pub: &str,
    issuer_account: &str,
    signing_kp: &KeyPair,
    permissions: Value,
) -> Result<String, AuthError> {
    let mut nats = permissions;
    let obj = nats
        .as_object_mut()
        .ok_or_else(|| AuthError::Jwt("permissions must be a JSON object".into()))?;
    obj.insert("type".into(), json!("user"));
    obj.insert("version".into(), json!(2));
    obj.insert("issuer_account".into(), json!(issuer_account));
    // User-level NATS limits: unlimited.
    obj.insert("subs".into(), json!(-1));
    obj.insert("data".into(), json!(-1));
    obj.insert("payload".into(), json!(-1));
    encode(name, user_pub, nats, signing_kp)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minted user JWT has three segments, the v2 header, and a recoverable `sub`/`iss`.
    #[test]
    fn user_jwt_structure_is_valid_v2() {
        let account = KeyPair::new_account();
        let signing = KeyPair::new_account();
        let user = KeyPair::new_user();
        let jwt = encode_user(
            "agent",
            &user.public_key(),
            &account.public_key(),
            &signing,
            json!({"pub": {"allow": ["a.b"]}, "sub": {"allow": ["_INBOX.>"]}}),
        )
        .unwrap();

        let segs: Vec<&str> = jwt.split('.').collect();
        assert_eq!(segs.len(), 3);
        let header = BASE64URL_NOPAD.decode(segs[0].as_bytes()).unwrap();
        assert_eq!(String::from_utf8(header).unwrap(), HEADER);
        let claims: Value =
            serde_json::from_slice(&BASE64URL_NOPAD.decode(segs[1].as_bytes()).unwrap()).unwrap();
        assert_eq!(claims["sub"], user.public_key());
        assert_eq!(claims["iss"], signing.public_key());
        assert_eq!(claims["nats"]["type"], "user");
        assert_eq!(claims["nats"]["version"], 2);
        assert_eq!(claims["nats"]["issuer_account"], account.public_key());
        assert_eq!(claims["nats"]["pub"]["allow"][0], "a.b");
        assert!(!claims["jti"].as_str().unwrap().is_empty());
    }
}
