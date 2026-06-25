//! The Hermes launch recipe: build the isolated profile and the [`LaunchSpec`] the manager spawns.
//! Port of Cotal `extensions/connector-hermes/src/launch.ts`.

use parler_core::{LaunchOpts, LaunchSpec};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Hermes API line this connector is written + pinned against. A different major.minor may move the
/// plugin/platform/hook signatures, so the launcher asserts and fails loudly.
pub const HERMES_PIN: &str = "0.16";

/// Orientation bootstrap appended to a persona's SOUL so the agent orients before acting.
pub const ORIENTATION_BOOTSTRAP: &str =
    "Before anything else, call `parler_orientation` to learn who you are, which channels you can \
read and post to, who else is present, and what tools you have. Re-check it anytime.";

/// Sanitize a string into a safe path/socket token (≤40 chars), matching Cotal's `tok`.
pub fn tok(s: &str) -> String {
    let t: String = s
        .trim()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '_' | '-') { c } else { '_' })
        .take(40)
        .collect();
    if t.is_empty() {
        "_".to_string()
    } else {
        t
    }
}

/// The bridge socket path the sidecar binds and the plugin connects to.
pub fn bridge_socket_path(space: &str, name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("parler-hermes-bridge-{}-{}.sock", tok(space), tok(name)))
}

/// A double-quoted YAML basic-string literal (escaped).
fn yaml_str(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Render the Parler-managed Hermes `config.yaml`: enable the `parler` plugin + platform, turn
/// approvals off (an autonomous spawned gateway has no human at the TUI), and set the model.
pub fn render_config(model: Option<&str>) -> String {
    let mut lines = vec![
        "# Parler-managed Hermes profile — regenerated each launch; do not edit.".to_string(),
        "plugins:".to_string(),
        "  enabled: [parler]".to_string(),
        "gateway:".to_string(),
        "  platforms:".to_string(),
        "    parler:".to_string(),
        "      enabled: true".to_string(),
        "approvals:".to_string(),
        "  mode: off".to_string(),
    ];
    if let Some(m) = model {
        lines.push(format!("model: {}", yaml_str(m)));
    }
    lines.join("\n") + "\n"
}

/// A persona → Hermes' SOUL.md identity file, with the orientation bootstrap appended.
pub fn render_soul(persona: &str) -> String {
    format!("{}\n\n{}\n", persona.trim(), ORIENTATION_BOOTSTRAP)
}

/// Write the isolated Hermes profile (config.yaml, and SOUL.md when a persona is set) under `home`.
/// (Copying the bundled plugin dir into `home/plugins/parler` is the manager's job at spawn — it
/// owns the packaged plugin path.)
pub fn setup_profile(
    home: &Path,
    model: Option<&str>,
    persona: Option<&str>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(home)?;
    std::fs::write(home.join("config.yaml"), render_config(model))?;
    if let Some(p) = persona {
        std::fs::write(home.join("SOUL.md"), render_soul(p))?;
    }
    Ok(())
}

/// Build the [`LaunchSpec`] the manager spawns to start a Hermes agent. The connector supervisor
/// (`parler-connect-hermes`) reads the `PARLER_*` env, sets up the profile + sidecar, and runs the
/// gateway as its child.
pub fn build_launch(opts: &LaunchOpts) -> LaunchSpec {
    let mut env: BTreeMap<String, String> = BTreeMap::new();
    env.insert("PARLER_SPACE".into(), opts.space.clone());
    env.insert("PARLER_NAME".into(), opts.name.clone());
    if let Some(role) = &opts.role {
        env.insert("PARLER_ROLE".into(), role.clone());
    }
    if let Some(servers) = &opts.servers {
        env.insert("PARLER_SERVERS".into(), servers.clone());
    }
    if let Some(creds) = &opts.creds {
        env.insert("PARLER_CREDS".into(), creds.clone());
    }
    if let Some(id) = &opts.id {
        env.insert("PARLER_ID".into(), id.clone());
    }
    if let Some(cfg) = &opts.config_path {
        env.insert("PARLER_AGENT_FILE".into(), cfg.clone());
    }
    if let Some(prompt) = &opts.prompt {
        env.insert("PARLER_PROMPT".into(), prompt.clone());
    }
    if opts.transcript {
        env.insert("PARLER_TRANSCRIPT".into(), "1".into());
    }
    if let Some(model) = &opts.model {
        // Each connector renders the model in its host form; Hermes reads HERMES_MODEL.
        env.insert("HERMES_MODEL".into(), model.clone());
    }
    LaunchSpec {
        command: "parler-connect-hermes".into(),
        args: Vec::new(),
        env,
        confirm: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tok_sanitizes_and_bounds() {
        assert_eq!(tok("my space"), "my_space");
        assert_eq!(tok("a/b.c"), "a_b_c");
        assert_eq!(tok("  "), "_");
        assert_eq!(tok(&"x".repeat(50)).len(), 40);
    }

    #[test]
    fn bridge_socket_path_uses_tokens() {
        let p = bridge_socket_path("main team", "alice/1");
        let s = p.to_string_lossy();
        assert!(s.contains("parler-hermes-bridge-main_team-alice_1.sock"), "got {s}");
    }

    #[test]
    fn config_enables_parler_and_optional_model() {
        let c = render_config(None);
        assert!(c.contains("enabled: [parler]"));
        assert!(c.contains("    parler:\n      enabled: true"));
        assert!(c.contains("approvals:\n  mode: off"));
        assert!(!c.contains("model:"));
        let c = render_config(Some("hermes-4"));
        assert!(c.contains("model: \"hermes-4\""));
    }

    #[test]
    fn soul_appends_orientation() {
        let s = render_soul("  You are a reviewer.  ");
        assert!(s.starts_with("You are a reviewer."));
        assert!(s.contains("parler_orientation"));
    }

    #[test]
    fn build_launch_maps_opts_to_env() {
        let opts = LaunchOpts {
            space: "main".into(),
            name: "alice".into(),
            role: Some("reviewer".into()),
            creds: Some("/tmp/a.creds".into()),
            config_path: Some("/p/.parler/agents/alice.md".into()),
            model: Some("hermes-4".into()),
            transcript: true,
            ..Default::default()
        };
        let spec = build_launch(&opts);
        assert_eq!(spec.command, "parler-connect-hermes");
        assert_eq!(spec.env["PARLER_SPACE"], "main");
        assert_eq!(spec.env["PARLER_NAME"], "alice");
        assert_eq!(spec.env["PARLER_ROLE"], "reviewer");
        assert_eq!(spec.env["PARLER_CREDS"], "/tmp/a.creds");
        assert_eq!(spec.env["PARLER_AGENT_FILE"], "/p/.parler/agents/alice.md");
        assert_eq!(spec.env["HERMES_MODEL"], "hermes-4");
        assert_eq!(spec.env["PARLER_TRANSCRIPT"], "1");
        // Unset optionals are absent, not empty.
        assert!(!spec.env.contains_key("PARLER_SERVERS"));
        assert!(!spec.env.contains_key("PARLER_PROMPT"));
    }

    #[test]
    fn setup_profile_writes_files() {
        let dir = tempfile::tempdir().unwrap();
        setup_profile(dir.path(), Some("hermes-4"), Some("You are alice.")).unwrap();
        let cfg = std::fs::read_to_string(dir.path().join("config.yaml")).unwrap();
        assert!(cfg.contains("enabled: [parler]"));
        let soul = std::fs::read_to_string(dir.path().join("SOUL.md")).unwrap();
        assert!(soul.contains("You are alice."));
    }
}
