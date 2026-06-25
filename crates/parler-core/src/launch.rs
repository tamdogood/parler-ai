//! Launch contracts. Port of the launch shapes in Cotal `connector.ts`.

use std::collections::BTreeMap;

/// Identity + mesh coordinates the manager hands a connector to launch an agent.
#[derive(Debug, Clone, Default)]
pub struct LaunchOpts {
    pub space: String,
    pub name: String,
    pub role: Option<String>,
    /// Stable agent id (the nkey public key). When set, the launched session adopts it as `card.id`.
    pub id: Option<String>,
    /// Path to a minted creds file (auth mode); absent when the mesh runs open.
    pub creds: Option<String>,
    pub servers: Option<String>,
    /// Path to an agent definition file (`.parler/agents/<name>.md`).
    pub config_path: Option<String>,
    /// Explicit model override (the `parler start --model <m>` flag).
    pub model: Option<String>,
    /// An initial message for the session to act on the moment it starts.
    pub prompt: Option<String>,
    /// Mirror this session's transcript so peers/observers can read what the agent did.
    pub transcript: bool,
    /// Operator MCP servers to share with this agent (`.mcp.json`-shaped, `${VAR}` refs intact).
    pub mcp_servers: BTreeMap<String, serde_json::Value>,
}

/// A recipe for starting an agent as a mesh node — command, args, and extra env.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LaunchSpec {
    pub command: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    /// Auto-clear a one-time spawn prompt: when this text appears in the agent's early output, the
    /// runtime presses Enter once so a supervised launch stays non-interactive.
    pub confirm: Option<String>,
}
