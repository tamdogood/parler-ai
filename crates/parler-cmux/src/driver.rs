//! The one place that knows the cmux CLI. Thin wrappers over `cmux <subcommand>` (the CLI talks to
//! the running cmux app over its Unix socket). Port of Cotal `extensions/cmux/src/driver.ts`.

use parler_core::RuntimeError;
use regex::Regex;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// Inside a cmux surface the CLI isn't on `$PATH`; cmux exports its absolute path here. Fall back to
/// `cmux` for non-bundled installs (e.g. a Homebrew cmux on PATH).
fn cmux_bin() -> String {
    std::env::var("CMUX_BUNDLED_CLI_PATH").unwrap_or_else(|_| "cmux".to_string())
}

fn uuid_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}")
            .unwrap()
    })
}

fn ws_ref_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"workspace:\d+").unwrap())
}

fn run(args: &[&str]) -> Result<String, RuntimeError> {
    let out = Command::new(cmux_bin())
        .args(args)
        .output()
        .map_err(|e| RuntimeError::Cli(format!("cmux: {e}")))?;
    if !out.status.success() {
        return Err(RuntimeError::Cli(format!(
            "cmux {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// True if a cmux app is reachable (`cmux ping`).
pub fn available() -> bool {
    Command::new(cmux_bin())
        .arg("ping")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// A terminal target — a workspace (tab) or a specific surface, by id/ref.
#[derive(Debug, Clone, Default)]
pub struct Target {
    pub workspace: Option<String>,
    pub surface: Option<String>,
}

fn target_args(t: &Target) -> Vec<String> {
    let mut a = Vec::new();
    if let Some(w) = &t.workspace {
        a.push("--workspace".to_string());
        a.push(w.clone());
    }
    if let Some(s) = &t.surface {
        a.push("--surface".to_string());
        a.push(s.clone());
    }
    a
}

/// cmux prints a UUID under `--id-format uuids`, but write ops confirm with `OK workspace:<n>` (a
/// short ref) — accept either.
pub(crate) fn parse_workspace_id(out: &str) -> Option<String> {
    uuid_re()
        .find(out)
        .map(|m| m.as_str().to_string())
        .or_else(|| ws_ref_re().find(out).map(|m| m.as_str().to_string()))
}

/// Open a new workspace (tab) with a declarative split layout (JSON). Returns the new workspace's
/// stable id so callers can later target or close it.
pub fn open_workspace(name: &str, layout: &str, focus: bool) -> Result<String, RuntimeError> {
    let out = run(&[
        "--id-format",
        "uuids",
        "new-workspace",
        "--name",
        name,
        "--focus",
        &focus.to_string(),
        "--layout",
        layout,
    ])?;
    parse_workspace_id(&out).ok_or_else(|| {
        RuntimeError::Cli(format!(
            "cmux new-workspace: couldn't read the new workspace id from \"{out}\""
        ))
    })
}

/// cmux exits non-zero with `not_found: Workspace not found` once the tab is already gone.
fn is_workspace_not_found(e: &RuntimeError) -> bool {
    matches!(e, RuntimeError::Cli(m) if m.contains("not_found: Workspace not found"))
}

/// Close a workspace (tab) by id/ref. Idempotent: closing an already-gone tab is a no-op.
pub fn close_workspace(workspace: &str) -> Result<(), RuntimeError> {
    match run(&["close-workspace", "--workspace", workspace]) {
        Ok(_) => Ok(()),
        Err(e) if is_workspace_not_found(&e) => Ok(()),
        Err(e) => Err(e),
    }
}

/// All open workspace lines (name + ref), or `[]` if cmux can't be reached.
pub fn list_workspaces() -> Vec<String> {
    run(&["list-workspaces"])
        .map(|o| {
            o.lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Workspace refs whose label is exactly `name` (matching the whole label keeps `parler-main` from
/// matching `parler-manager`). cmux lists tabs as `[*] <ref>  [glyph] <label> [\[selected\]]`.
pub(crate) fn parse_workspace_refs(lines: &[String], name: &str) -> Vec<String> {
    let mut refs = Vec::new();
    for line in lines {
        let r = ws_ref_re()
            .find(line)
            .map(|m| m.as_str().to_string())
            .or_else(|| uuid_re().find(line).map(|m| m.as_str().to_string()));
        let Some(r) = r else { continue };
        let after = line.find(&r).map(|i| i + r.len()).unwrap_or(0);
        let label = line[after..].trim();
        let label = label.strip_suffix("[selected]").unwrap_or(label).trim();
        if label == name || label.ends_with(&format!(" {name}")) {
            refs.push(r);
        }
    }
    refs
}

pub fn workspace_refs(name: &str) -> Vec<String> {
    parse_workspace_refs(&list_workspaces(), name)
}

/// Type text into a terminal surface (the focused one, or a targeted background tab).
pub fn send(text: &str, target: &Target) -> Result<(), RuntimeError> {
    let mut args = vec!["send".to_string()];
    args.extend(target_args(target));
    args.push("--".to_string());
    args.push(text.to_string());
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&argv).map(|_| ())
}

/// Send a key press (e.g. `enter`) to a terminal surface.
pub fn send_key(key: &str, target: &Target) -> Result<(), RuntimeError> {
    let mut args = vec!["send-key".to_string()];
    args.extend(target_args(target));
    args.push("--".to_string());
    args.push(key.to_string());
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    run(&argv).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_workspace_id_accepts_uuid_or_ref() {
        assert_eq!(
            parse_workspace_id("OK workspace:5"),
            Some("workspace:5".to_string())
        );
        let u = "3f2504e0-4f89-41d3-9a0c-0305e82c3301";
        assert_eq!(parse_workspace_id(&format!("created {u}")), Some(u.to_string()));
        assert_eq!(parse_workspace_id("garbage"), None);
    }

    #[test]
    fn parse_workspace_refs_matches_whole_label() {
        let lines = vec![
            "[*] workspace:1  📁 parler-main".to_string(),
            "[ ] workspace:2  📁 parler-manager".to_string(),
            "[ ] workspace:3  📁 parler-main [selected]".to_string(),
        ];
        // "parler-main" must NOT match "parler-manager".
        assert_eq!(
            parse_workspace_refs(&lines, "parler-main"),
            vec!["workspace:1".to_string(), "workspace:3".to_string()]
        );
        assert_eq!(
            parse_workspace_refs(&lines, "parler-manager"),
            vec!["workspace:2".to_string()]
        );
    }
}
