//! Terminal-multiplexer integration contract. Port of Cotal `terminal.ts`.
//!
//! NOTE: this is **not** part of the Parler wire protocol — it's a host-side integration contract
//! for driving a terminal multiplexer (open/close tabs). Nothing here references a mesh concept.

use crate::error::RuntimeError;
use std::collections::BTreeMap;

/// One terminal pane in a [`Tab`]: an argv `command` (+ `args`), optional `cwd`/`env`. The backend is
/// responsible for shell-quoting, so callers never build shell strings. `confirm` asks the backend to
/// auto-accept a one-time confirmation prompt the command may show on start.
#[derive(Debug, Clone, Default)]
pub struct Pane {
    pub command: String,
    pub args: Vec<String>,
    pub cwd: Option<String>,
    pub env: BTreeMap<String, String>,
    pub confirm: bool,
}

/// Split orientation — **normative**, so every backend renders the same shape.
/// - `Vertical` — a vertical divider (side-by-side left/right columns).
/// - `Horizontal` — a horizontal divider (stacked top/bottom rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Vertical,
    Horizontal,
}

impl SplitDirection {
    pub fn as_str(self) -> &'static str {
        match self {
            SplitDirection::Vertical => "vertical",
            SplitDirection::Horizontal => "horizontal",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Split {
    pub direction: SplitDirection,
    /// The **first** pane's fraction of the tab (the second gets `1 - ratio`).
    pub ratio: f64,
}

/// A tab to open: a single pane, or several panes split along `direction` at `ratio`.
#[derive(Debug, Clone, Default)]
pub struct Tab {
    pub panes: Vec<Pane>,
    pub split: Option<Split>,
}

/// A terminal-multiplexer surface an integration drives. Tabs are described by a backend-agnostic
/// [`Tab`], so the provider owns all backend-specific layout construction.
pub trait TerminalLayout {
    fn name(&self) -> &str;
    /// Whether the backend is reachable right now (e.g. the cmux app is running).
    fn available(&self) -> bool;
    /// Open a tab labelled `label`, laid out per [`Tab`]; returns its ref (id).
    fn open(&self, label: &str, tab: &Tab, focus: bool) -> Result<String, RuntimeError>;
    /// Close a previously opened tab by ref.
    fn close(&self, reference: &str) -> Result<(), RuntimeError>;
    /// Refs of every open tab labelled `label` (dead tabs may linger).
    fn refs(&self, label: &str) -> Vec<String>;
}
