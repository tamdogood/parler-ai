//! The Hermes adapter bridge — wire protocol + the serial ack-on-surface state machine.
//!
//! Port of Cotal `extensions/connector-hermes/src/bridge.ts`. The bridge is a local unix-socket
//! channel between the in-gateway **Python plugin** and this Rust **sidecar**: the sidecar *pushes*
//! inbound mesh messages into a live gateway turn and the plugin routes turn replies + `parler_*`
//! tool calls back out.
//!
//! Wire format: newline-delimited JSON, both directions.
//! - Python → sidecar: [`InFrame`] (`subscribe` / `delivered` / `reply` / `tool`)
//! - sidecar → Python: [`OutFrame`] (`incoming` / `tool_result`)
//!
//! Delivery is **serial + ack-on-surface**: push the oldest buffered message, wait for the adapter's
//! `delivered`, ack exactly that message, then push the next. A crash before `delivered` redelivers.
//! That ordering logic is [`BridgeState`] — a pure state machine, unit-tested below; `serve.rs` wires
//! it to a real socket + a [`crate::MeshHandle`].

use parler_protocol::MessageKind;
use serde::{Deserialize, Serialize};

/// The inbox item, flattened for the Python side (its `handle_message` builds a turn from it). Wire
/// keys are camelCase to match the Cotal bridge exactly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WireItem {
    pub id: String,
    pub ts: i64,
    pub kind: MessageKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(rename = "fromId")]
    pub from_id: String,
    #[serde(rename = "fromName")]
    pub from_name: String,
    #[serde(rename = "fromRole", skip_serializing_if = "Option::is_none")]
    pub from_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
    #[serde(rename = "mentionsMe")]
    pub mentions_me: bool,
    pub text: String,
    #[serde(rename = "replyTo", skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    #[serde(rename = "contextId", skip_serializing_if = "Option::is_none")]
    pub context_id: Option<String>,
}

/// A buffered inbound message as the [`crate::MeshHandle`] yields it (the sidecar-internal shape).
#[derive(Debug, Clone, PartialEq)]
pub struct InboxItem {
    pub id: String,
    pub ts: i64,
    pub kind: MessageKind,
    pub channel: Option<String>,
    pub service: Option<String>,
    pub from_id: String,
    pub from_name: String,
    pub from_role: Option<String>,
    pub mentions: Option<Vec<String>>,
    pub mentions_me: bool,
    pub text: String,
    pub reply_to: Option<String>,
    pub context_id: Option<String>,
}

impl InboxItem {
    pub fn to_wire(&self) -> WireItem {
        WireItem {
            id: self.id.clone(),
            ts: self.ts,
            kind: self.kind,
            channel: self.channel.clone(),
            service: self.service.clone(),
            from_id: self.from_id.clone(),
            from_name: self.from_name.clone(),
            from_role: self.from_role.clone(),
            mentions: self.mentions.clone(),
            mentions_me: self.mentions_me,
            text: self.text.clone(),
            reply_to: self.reply_to.clone(),
            context_id: self.context_id.clone(),
        }
    }
}

/// Reply routing target the adapter derives from a turn's chat id.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplyTarget {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    /// Peer instance id (or name) for a DM/anycast reply.
    #[serde(rename = "peerId", default, skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,
}

/// Python → sidecar frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum InFrame {
    /// Start receiving inbound pushes.
    Subscribe,
    /// Turn accepted msg `id` → ack it on the stream.
    Delivered { id: String },
    /// Route a turn's reply back to its origin.
    Reply {
        #[serde(default)]
        target: ReplyTarget,
        #[serde(default)]
        text: String,
    },
    /// Invoke a `parler_*` tool (full shared surface).
    Tool {
        id: String,
        name: String,
        #[serde(default)]
        args: serde_json::Value,
    },
}

/// sidecar → Python frames.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "snake_case")]
pub enum OutFrame {
    /// Push one buffered mesh message.
    Incoming { msg: WireItem },
    /// Reply to a [`InFrame::Tool`] request.
    ToolResult {
        id: String,
        ok: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<String>,
        #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
}

/// The result of running a `parler_*` tool.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    pub text: String,
    pub is_error: bool,
}

/// A read-only view of the front of the agent's inbox — what [`BridgeState`] needs to decide the next
/// push and whether a `delivered` still matches the front.
pub trait InboxView {
    fn peek_front(&self) -> Option<InboxItem>;
}

/// The serial ack-on-surface ordering, isolated from any IO so it can be tested directly.
///
/// One message is in flight at a time (`awaiting`): [`BridgeState::pump`] picks the front and marks
/// it awaiting; the adapter's `delivered` for that exact id releases it (acking the front iff it is
/// still the front — a large ambient burst during a long turn can evict our in-flight item, in which
/// case the overflow already acked it and we must not mis-ack a newer message).
#[derive(Debug, Default)]
pub struct BridgeState {
    subscribed: bool,
    awaiting: Option<String>,
}

impl BridgeState {
    pub fn new() -> Self {
        Self::default()
    }

    /// A (re)subscribe supersedes any previous one and resets the in-flight marker.
    pub fn on_subscribe(&mut self, inbox: &dyn InboxView) -> Option<WireItem> {
        self.subscribed = true;
        self.awaiting = None;
        self.pump(inbox)
    }

    /// The adapter surfaced `id` into a turn. Returns `true` iff the caller should ack (drain) the
    /// front. Clears the in-flight marker either way so a later [`Self::pump`] can advance.
    pub fn on_delivered(&mut self, id: &str, inbox: &dyn InboxView) -> bool {
        if self.awaiting.as_deref() != Some(id) {
            return false;
        }
        let front_is_ours = inbox.peek_front().map(|i| i.id).as_deref() == Some(id);
        self.awaiting = None;
        front_is_ours
    }

    /// Push the oldest buffered message, if subscribed and nothing is already in flight.
    pub fn pump(&mut self, inbox: &dyn InboxView) -> Option<WireItem> {
        if !self.subscribed || self.awaiting.is_some() {
            return None;
        }
        let front = inbox.peek_front()?;
        self.awaiting = Some(front.id.clone());
        Some(front.to_wire())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    fn item(id: &str) -> InboxItem {
        InboxItem {
            id: id.into(),
            ts: 0,
            kind: MessageKind::Channel,
            channel: Some("general".into()),
            service: None,
            from_id: "U1".into(),
            from_name: "alice".into(),
            from_role: None,
            mentions: None,
            mentions_me: false,
            text: "hi".into(),
            reply_to: None,
            context_id: None,
        }
    }

    /// A mock inbox whose front is index 0; tests drain by popping the front.
    struct MockInbox(RefCell<Vec<InboxItem>>);
    impl MockInbox {
        fn new(items: Vec<InboxItem>) -> Self {
            MockInbox(RefCell::new(items))
        }
        fn drain_one(&self) {
            let mut v = self.0.borrow_mut();
            if !v.is_empty() {
                v.remove(0);
            }
        }
    }
    impl InboxView for MockInbox {
        fn peek_front(&self) -> Option<InboxItem> {
            self.0.borrow().first().cloned()
        }
    }

    #[test]
    fn serial_pump_acks_and_advances_in_order() {
        let inbox = MockInbox::new(vec![item("a"), item("b")]);
        let mut st = BridgeState::new();

        // subscribe pushes the front (a).
        assert_eq!(st.on_subscribe(&inbox).map(|w| w.id), Some("a".into()));
        // nothing new pushes while a is in flight.
        assert!(st.pump(&inbox).is_none());

        // delivered(a): front is still a → ack; caller drains; pump pushes b.
        assert!(st.on_delivered("a", &inbox));
        inbox.drain_one();
        assert_eq!(st.pump(&inbox).map(|w| w.id), Some("b".into()));

        // delivered(b): ack, drain, then nothing left.
        assert!(st.on_delivered("b", &inbox));
        inbox.drain_one();
        assert!(st.pump(&inbox).is_none());
    }

    #[test]
    fn delivered_wrong_id_is_ignored() {
        let inbox = MockInbox::new(vec![item("a")]);
        let mut st = BridgeState::new();
        assert_eq!(st.on_subscribe(&inbox).map(|w| w.id), Some("a".into()));
        // A stale/foreign id neither acks nor releases the in-flight a.
        assert!(!st.on_delivered("zzz", &inbox));
        assert!(st.pump(&inbox).is_none()); // still awaiting a
    }

    #[test]
    fn evicted_in_flight_item_is_not_mis_acked() {
        // a was pushed and is awaiting, but an overflow evicted it: the front is now b.
        let inbox = MockInbox::new(vec![item("b")]);
        let mut st = BridgeState::new();
        st.awaiting = Some("a".into());
        st.subscribed = true;
        // delivered(a): front is b, not a → do NOT ack (the overflow already acked a).
        assert!(!st.on_delivered("a", &inbox));
        // but the marker is released, so the next pump advances to b.
        assert_eq!(st.pump(&inbox).map(|w| w.id), Some("b".into()));
    }

    #[test]
    fn nothing_pushes_before_subscribe() {
        let inbox = MockInbox::new(vec![item("a")]);
        let mut st = BridgeState::new();
        assert!(st.pump(&inbox).is_none());
    }

    #[test]
    fn frames_round_trip_with_exact_wire_tags() {
        // Python → sidecar.
        let sub: InFrame = serde_json::from_str(r#"{"t":"subscribe"}"#).unwrap();
        assert_eq!(sub, InFrame::Subscribe);
        let del: InFrame = serde_json::from_str(r#"{"t":"delivered","id":"m1"}"#).unwrap();
        assert_eq!(del, InFrame::Delivered { id: "m1".into() });
        let rep: InFrame =
            serde_json::from_str(r#"{"t":"reply","target":{"peerId":"U2"},"text":"hi"}"#).unwrap();
        assert_eq!(
            rep,
            InFrame::Reply {
                target: ReplyTarget { channel: None, peer_id: Some("U2".into()) },
                text: "hi".into()
            }
        );
        let tool: InFrame =
            serde_json::from_str(r#"{"t":"tool","id":"r1","name":"parler_send","args":{"text":"x"}}"#)
                .unwrap();
        assert!(matches!(tool, InFrame::Tool { .. }));

        // sidecar → Python.
        let inc = OutFrame::Incoming { msg: item("a").to_wire() };
        let j = serde_json::to_value(&inc).unwrap();
        assert_eq!(j["t"], "incoming");
        assert_eq!(j["msg"]["kind"], "channel");
        assert_eq!(j["msg"]["fromId"], "U1");
        assert_eq!(j["msg"]["mentionsMe"], false);

        let res = OutFrame::ToolResult {
            id: "r1".into(),
            ok: true,
            text: Some("done".into()),
            is_error: Some(false),
            error: None,
        };
        let j = serde_json::to_value(&res).unwrap();
        assert_eq!(j["t"], "tool_result");
        assert_eq!(j["isError"], false);
        assert!(j.get("error").is_none());
    }
}
