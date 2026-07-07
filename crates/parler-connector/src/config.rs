//! Local agent state: the hub URL, the agent's display name/role, and its nkey identity.
//!
//! Persisted to `$PARLER_HOME/config.json` (default `~/.parler/config.json`) with `0600` perms —
//! it holds the nkey **seed** (the private half of the identity), which never goes on the wire.

use anyhow::{Context, Result};
use parler_auth::{new_identity, Identity};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Serialize, Deserialize)]
struct ConfigFile {
    hub_url: String,
    id: String,
    seed: String,
    name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    role: Option<String>,
}

// `seed` is private key material — keep it out of any `{:?}` / log line (mirrors `Identity`'s Debug).
impl std::fmt::Debug for ConfigFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigFile")
            .field("hub_url", &self.hub_url)
            .field("id", &self.id)
            .field("seed", &"<redacted>")
            .field("name", &self.name)
            .field("role", &self.role)
            .finish()
    }
}

/// The agent's local configuration + identity.
#[derive(Debug, Clone)]
pub struct Config {
    pub hub_url: String,
    pub identity: Identity,
    pub name: String,
    pub role: Option<String>,
}

/// The Parler Protocol home directory: `$PARLER_HOME`, else `~/.parler`.
pub fn home_dir() -> PathBuf {
    if let Some(p) = std::env::var("PARLER_HOME").ok().filter(|p| !p.is_empty()) {
        return expand_tilde(&p);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".parler")
}

/// Expand a leading `~` (or `~/`) to `$HOME`. The shell only expands `~` in *unquoted* argv, so a
/// documented `-e PARLER_HOME=~/.parler-bob` (issue #112) arrives here literally and would otherwise
/// create a `./~` directory. A bare `~` or `~/...` expands; a `~user` form is left untouched (we
/// don't resolve other users' homes).
fn expand_tilde(p: &str) -> PathBuf {
    expand_tilde_with(p, std::env::var("HOME").ok().map(PathBuf::from))
}

/// The pure core of [`expand_tilde`], with `$HOME` injected so it's testable without touching the
/// process environment.
fn expand_tilde_with(p: &str, home: Option<PathBuf>) -> PathBuf {
    match home {
        Some(home) if p == "~" => home,
        Some(home) if p.starts_with("~/") => home.join(&p[2..]),
        _ => PathBuf::from(p),
    }
}

fn config_path() -> PathBuf {
    home_dir().join("config.json")
}

impl Config {
    /// Create a fresh identity + config (not yet saved).
    pub fn create(hub_url: impl Into<String>, name: impl Into<String>, role: Option<String>) -> Result<Config> {
        Ok(Config {
            hub_url: hub_url.into(),
            identity: new_identity()?,
            name: name.into(),
            role,
        })
    }

    /// Load the saved config, or a helpful error pointing at `parler init`.
    pub fn load() -> Result<Config> {
        let path = config_path();
        let data = std::fs::read_to_string(&path).with_context(|| {
            format!("no Parler Protocol identity at {} — run `parler init` first", path.display())
        })?;
        let f: ConfigFile = serde_json::from_str(&data).context("parsing config.json")?;
        Ok(Config {
            hub_url: f.hub_url,
            identity: Identity { id: f.id, seed: f.seed },
            name: f.name,
            role: f.role,
        })
    }

    pub fn exists() -> bool {
        config_path().exists()
    }

    /// Persist to `$PARLER_HOME/config.json`, owner-only (`0600`) — it holds the private seed, so the
    /// write is atomic (temp file + rename) and never leaves the seed at the default umask.
    pub fn save(&self) -> Result<()> {
        let f = ConfigFile {
            hub_url: self.hub_url.clone(),
            id: self.identity.id.clone(),
            seed: self.identity.seed.clone(),
            name: self.name.clone(),
            role: self.role.clone(),
        };
        let path = config_path();
        let body = serde_json::to_string_pretty(&f)?;
        parler_auth::write_private_file(&path, body.as_bytes())
            .with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }

    /// Delete `$PARLER_HOME/config.json` if present — used to roll back a freshly-minted identity
    /// that could never reach its hub, so a first-run network failure doesn't strand an identity on
    /// disk that never registered anywhere. A missing file is not an error (idempotent).
    pub fn remove() -> Result<()> {
        match std::fs::remove_file(config_path()) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e).with_context(|| format!("removing {}", config_path().display())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_tilde_resolves_leading_home() {
        let home = Some(PathBuf::from("/home/bob"));
        assert_eq!(expand_tilde_with("~/.parler-bob", home.clone()), PathBuf::from("/home/bob/.parler-bob"));
        assert_eq!(expand_tilde_with("~", home.clone()), PathBuf::from("/home/bob"));
        // Absolute paths and mid-string tildes are untouched; ~user is left alone.
        assert_eq!(expand_tilde_with("/tmp/x", home.clone()), PathBuf::from("/tmp/x"));
        assert_eq!(expand_tilde_with("~alice/x", home.clone()), PathBuf::from("~alice/x"));
        // No HOME → leave the literal (don't fabricate a path).
        assert_eq!(expand_tilde_with("~/x", None), PathBuf::from("~/x"));
    }

    #[test]
    fn remove_deletes_config_and_is_idempotent() {
        // std-only temp dir (parler-connector has no tempfile dev-dep) — unique per run.
        let dir = std::env::temp_dir().join(format!("parler-cfg-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let prev = std::env::var("PARLER_HOME").ok();
        std::env::set_var("PARLER_HOME", &dir);

        // Removing when nothing is there is a no-op, not an error.
        assert!(Config::remove().is_ok());

        Config::create("ws://h", "bob", None).unwrap().save().unwrap();
        assert!(Config::exists());
        assert!(Config::remove().is_ok());
        assert!(!Config::exists(), "remove() should delete the on-disk identity");
        // Second removal is still fine (idempotent).
        assert!(Config::remove().is_ok());

        match prev {
            Some(p) => std::env::set_var("PARLER_HOME", p),
            None => std::env::remove_var("PARLER_HOME"),
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
