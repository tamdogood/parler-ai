//! The cmux `Runtime` + `TerminalLayout`. Port of Cotal `extensions/cmux/src/runtime.ts`.
//!
//! Each spawned agent gets its own new cmux tab (workspace) so teammates get room. Launch goes
//! through [`pane_command`] (a temp bash script) to sidestep all shell quoting. Like tmux you watch
//! it natively, so `attach()` errors — but teardown is real (we keep the tab id to drive + close it).

use crate::driver::{self, Target};
use parler_core::{
    AgentHandle, AgentStatus, AttachSession, LaunchSpec, Pane, Runtime, RuntimeError, Tab,
    TerminalLayout,
};
use serde_json::json;
use std::time::Duration;

/// Grace window for a clean exit before a graceful stop force-closes the tab.
const GRACE_MS: u64 = 1_500;

/// Background snippet that auto-accepts a one-time confirm prompt by pressing Enter on the pane's own
/// cmux surface a few times. Gated on the cmux env vars so it's a no-op off cmux.
const ENTER_LOOP: &str = "[ -n \"$CMUX_SURFACE_ID\" ] && [ -n \"$CMUX_BUNDLED_CLI_PATH\" ] && \
( for _ in 1 2 3 4 5; do sleep 1; \"$CMUX_BUNDLED_CLI_PATH\" send-key --surface \"$CMUX_SURFACE_ID\" enter >/dev/null 2>&1; done ) &";

fn shell_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// cmux can't run a command in a fresh surface directly, and panes start under a login shell before
/// bash — so we write each pane's launch as a temp bash script and point the tab at it (callers pass
/// argv, never shell strings). `login` runs it as `bash -l` so the user's PATH is present. `isolate`
/// adds `env -i` so a spawned agent inherits ONLY the connector-declared env, not the cmux server's.
fn pane_command(pane: &Pane, key: &str, login: bool, isolate: bool) -> Result<String, RuntimeError> {
    let mut parts: Vec<String> = pane
        .env
        .iter()
        .map(|(k, v)| format!("{k}={}", shell_quote(v)))
        .collect();
    parts.push(shell_quote(&pane.command));
    parts.extend(pane.args.iter().map(|a| shell_quote(a)));
    let cmd = parts.join(" ");
    let cd = pane
        .cwd
        .as_ref()
        .map(|c| format!("cd {}\n", shell_quote(c)))
        .unwrap_or_default();
    let confirm = if pane.confirm {
        format!("{ENTER_LOOP}\n")
    } else {
        String::new()
    };
    let isolate_flag = if isolate { "-i " } else { "" };
    let script = format!("#!/usr/bin/env bash\n{cd}{confirm}exec env {isolate_flag}{cmd}\n");

    let safe_key: String = key
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-') { c } else { '_' })
        .collect();
    let path = std::env::temp_dir().join(format!("parler-pane-{safe_key}.sh"));
    std::fs::write(&path, &script).map_err(|e| RuntimeError::Io(e.to_string()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| RuntimeError::Io(e.to_string()))?;
    }
    let login_flag = if login { "-l " } else { "" };
    Ok(format!("bash {login_flag}{}", path.display()))
}

/// A single-terminal pane node in cmux's layout JSON.
fn surface(command: &str) -> serde_json::Value {
    json!({ "pane": { "surfaces": [ { "type": "terminal", "command": command } ] } })
}

/// Translate a backend-agnostic [`Tab`] into a cmux layout JSON string — the one place that knows
/// cmux's layout shape. One pane → a bare terminal; several → a split (direction + ratio).
fn cmux_layout(label: &str, tab: &Tab) -> Result<String, RuntimeError> {
    let mut nodes = Vec::with_capacity(tab.panes.len());
    for (i, p) in tab.panes.iter().enumerate() {
        let cmd = pane_command(p, &format!("{label}-{i}"), true, false)?;
        nodes.push(surface(&cmd));
    }
    if nodes.len() == 1 && tab.split.is_none() {
        return Ok(nodes.into_iter().next().unwrap().to_string());
    }
    let split = tab.split.ok_or_else(|| {
        RuntimeError::Cli(format!(
            "cmux layout \"{label}\": {} panes need a split (direction + ratio)",
            nodes.len()
        ))
    })?;
    Ok(json!({
        "direction": split.direction.as_str(),
        "split": split.ratio,
        "children": nodes,
    })
    .to_string())
}

/// Spawns each agent into its own new cmux tab.
#[derive(Debug, Default, Clone, Copy)]
pub struct CmuxRuntime;

impl CmuxRuntime {
    pub fn new() -> Self {
        CmuxRuntime
    }
}

impl Runtime for CmuxRuntime {
    fn kind(&self) -> &str {
        "cmux"
    }

    fn spawn(
        &self,
        name: &str,
        spec: &LaunchSpec,
        cwd: &str,
    ) -> Result<Box<dyn AgentHandle>, RuntimeError> {
        // `name` becomes a temp-script key and a `parler-<name>` tab id — keep it a bare token.
        if !name_is_safe(name) {
            return Err(RuntimeError::UnsafeName {
                kind: "cmux".into(),
                name: name.into(),
            });
        }
        if !driver::available() {
            return Err(RuntimeError::Unavailable("cmux".into()));
        }
        let pane = Pane {
            command: spec.command.clone(),
            args: spec.args.clone(),
            cwd: Some(cwd.to_string()),
            env: spec.env.clone(),
            confirm: spec.confirm.is_some(),
        };
        // isolate: a spawned agent gets ONLY the connector-declared env (env-bleed mitigation).
        let command = pane_command(&pane, &format!("spawn-{name}"), false, true)?;
        let workspace =
            driver::open_workspace(&format!("parler-{name}"), &surface(&command).to_string(), false)?;
        Ok(Box::new(CmuxHandle {
            name: name.to_string(),
            workspace,
        }))
    }
}

struct CmuxHandle {
    name: String,
    workspace: String,
}

impl CmuxHandle {
    fn target(&self) -> Target {
        Target {
            workspace: Some(self.workspace.clone()),
            surface: None,
        }
    }
}

impl AgentHandle for CmuxHandle {
    fn name(&self) -> &str {
        &self.name
    }
    fn kind(&self) -> &str {
        "cmux"
    }
    fn status(&self) -> AgentStatus {
        AgentStatus::Running
    }
    fn stop(&self, graceful: bool) -> Result<(), RuntimeError> {
        if !graceful {
            return driver::close_workspace(&self.workspace);
        }
        // Graceful: type `/exit` so the session shuts down cleanly (its SessionEnd hook leaves the
        // mesh), then close the now-idle tab after a grace window regardless.
        let _ = driver::send("/exit", &self.target());
        let _ = driver::send_key("enter", &self.target());
        let ws = self.workspace.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(GRACE_MS));
            if let Err(e) = driver::close_workspace(&ws) {
                eprintln!("cmux runtime: failed to close tab: {e}");
            }
        });
        Ok(())
    }
    fn interrupt(&self) -> Result<(), RuntimeError> {
        driver::send_key("ctrl+c", &self.target())
    }
    fn attach(&self) -> Result<Box<dyn AttachSession>, RuntimeError> {
        Err(RuntimeError::NotSupported(format!(
            "switch to the \"parler-{}\" cmux tab to watch it",
            self.name
        )))
    }
}

/// The cmux terminal-layout provider — lets a caller (e.g. `parler setup`) open/close cmux tabs from
/// a backend-agnostic [`Tab`].
#[derive(Debug, Default, Clone, Copy)]
pub struct CmuxTerminal;

impl TerminalLayout for CmuxTerminal {
    fn name(&self) -> &str {
        "cmux"
    }
    fn available(&self) -> bool {
        driver::available()
    }
    fn open(&self, label: &str, tab: &Tab, focus: bool) -> Result<String, RuntimeError> {
        driver::open_workspace(label, &cmux_layout(label, tab)?, focus)
    }
    fn close(&self, reference: &str) -> Result<(), RuntimeError> {
        driver::close_workspace(reference)
    }
    fn refs(&self, label: &str) -> Vec<String> {
        driver::workspace_refs(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parler_core::{Split, SplitDirection};

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("plain"), "'plain'");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn pane_command_writes_isolated_script() {
        let pane = Pane {
            command: "claude".into(),
            args: vec!["--model".into(), "opus".into()],
            cwd: Some("/work/dir".into()),
            env: [("PARLER_SPACE".to_string(), "main".to_string())]
                .into_iter()
                .collect(),
            confirm: false,
        };
        let cmd = pane_command(&pane, "spawn-alice", false, true).unwrap();
        assert!(cmd.starts_with("bash "), "got: {cmd}");
        assert!(!cmd.contains("bash -l "), "login flag should be off: {cmd}");
        let path = cmd.strip_prefix("bash ").unwrap();
        let script = std::fs::read_to_string(path).unwrap();
        assert!(script.contains("exec env -i "), "isolate missing: {script}");
        assert!(script.contains("PARLER_SPACE='main'"));
        assert!(script.contains("'claude' '--model' 'opus'"));
        assert!(script.contains("cd '/work/dir'"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn pane_command_login_and_confirm() {
        let pane = Pane {
            command: "echo".into(),
            confirm: true,
            ..Default::default()
        };
        let cmd = pane_command(&pane, "setup-0", true, false).unwrap();
        assert!(cmd.starts_with("bash -l "), "login flag should be on: {cmd}");
        let path = cmd.strip_prefix("bash -l ").unwrap();
        let script = std::fs::read_to_string(path).unwrap();
        assert!(script.contains("CMUX_SURFACE_ID"), "confirm loop missing");
        assert!(script.contains("exec env 'echo'"), "no isolate; got: {script}");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn layout_single_pane_is_bare_surface() {
        let tab = Tab {
            panes: vec![Pane {
                command: "echo".into(),
                ..Default::default()
            }],
            split: None,
        };
        let layout = cmux_layout("solo", &tab).unwrap();
        let v: serde_json::Value = serde_json::from_str(&layout).unwrap();
        assert_eq!(v["pane"]["surfaces"][0]["type"], "terminal");
        assert!(v.get("children").is_none());
    }

    #[test]
    fn layout_two_panes_need_a_split() {
        let two = vec![
            Pane { command: "a".into(), ..Default::default() },
            Pane { command: "b".into(), ..Default::default() },
        ];
        // No split → error.
        assert!(cmux_layout("x", &Tab { panes: two.clone(), split: None }).is_err());
        // With split → a children node carrying direction + ratio.
        let tab = Tab {
            panes: two,
            split: Some(Split { direction: SplitDirection::Vertical, ratio: 0.5 }),
        };
        let v: serde_json::Value = serde_json::from_str(&cmux_layout("x", &tab).unwrap()).unwrap();
        assert_eq!(v["direction"], "vertical");
        assert_eq!(v["split"], 0.5);
        assert_eq!(v["children"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn spawn_rejects_unsafe_name() {
        let rt = CmuxRuntime::new();
        let err = match rt.spawn("../evil", &LaunchSpec::default(), "/tmp") {
            Err(e) => e,
            Ok(_) => panic!("expected an unsafe-name error"),
        };
        assert!(matches!(err, RuntimeError::UnsafeName { .. }));
    }
}
