//! Receiver-side attention policy shared by MCP hooks and the optional local supervisor.
//!
//! The hub persists every authorized message and the room cursor remains authoritative. This module
//! only answers whether a newly noticed message should wake a host now, wait in its durable backlog,
//! or be intentionally discarded by a muted receiver.

use parler_protocol::{Attention, DispatchRef, HandoffRef, RoomKind, StoredMessage};
use serde::{Deserialize, Serialize};

/// A persisted, local interruption preference. Peers may observe the global mode through presence,
/// but room overrides and the actual wake decision stay on the receiving device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttentionPolicy {
    #[serde(default)]
    pub mode: Attention,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub quiet_rooms: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub muted_rooms: Vec<String>,
}

impl Default for AttentionPolicy {
    fn default() -> Self {
        AttentionPolicy { mode: Attention::Open, quiet_rooms: Vec::new(), muted_rooms: Vec::new() }
    }
}

/// What an adapter should do after it notices a message. `Hold` leaves the room cursor untouched so
/// the message remains available when attention opens; `Drop` means intentionally pull + acknowledge
/// without surfacing it, the explicit semantics of a muted room.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionDecision {
    Wake,
    Hold,
    Drop,
}

/// A per-room override accepted by the CLI/MCP configuration surface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomAttention {
    Quiet,
    Muted,
    Inherit,
}

impl RoomAttention {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "quiet" => Some(RoomAttention::Quiet),
            "muted" => Some(RoomAttention::Muted),
            "inherit" | "open" | "default" => Some(RoomAttention::Inherit),
            _ => None,
        }
    }
}

impl AttentionPolicy {
    /// Update one room override, keeping the two persisted lists mutually exclusive and stable.
    pub fn set_room(&mut self, room: &str, mode: RoomAttention) {
        self.quiet_rooms.retain(|r| r != room);
        self.muted_rooms.retain(|r| r != room);
        match mode {
            RoomAttention::Quiet => self.quiet_rooms.push(room.to_string()),
            RoomAttention::Muted => self.muted_rooms.push(room.to_string()),
            RoomAttention::Inherit => {}
        }
        self.quiet_rooms.sort();
        self.quiet_rooms.dedup();
        self.muted_rooms.sort();
        self.muted_rooms.dedup();
    }

    /// Decide how to treat one peer message. `role` is the current worker role when one is running;
    /// it may differ from the display role in the agent config.
    pub fn decide(
        &self,
        room: &str,
        room_kind: RoomKind,
        message: &StoredMessage,
        name: &str,
        role: Option<&str>,
    ) -> AttentionDecision {
        if self.muted_rooms.iter().any(|r| r == room) {
            return AttentionDecision::Drop;
        }
        let handoff = message
            .parts
            .iter()
            .filter_map(HandoffRef::from_part)
            .any(|handoff| handoff.is_for(name, role));
        let assigned = message.parts.iter().filter_map(DispatchRef::from_part).any(|dispatch| {
            role.is_some_and(|worker_role| dispatch.role.eq_ignore_ascii_case(worker_role))
        });
        let direct = room_kind == RoomKind::Dm || handoff || assigned;
        if self.quiet_rooms.iter().any(|r| r == room) {
            return if direct { AttentionDecision::Wake } else { AttentionDecision::Hold };
        }
        match self.mode {
            Attention::Open => AttentionDecision::Wake,
            Attention::Dnd => {
                if direct {
                    AttentionDecision::Wake
                } else {
                    AttentionDecision::Hold
                }
            }
            Attention::Focus => {
                if handoff || assigned {
                    AttentionDecision::Wake
                } else {
                    AttentionDecision::Hold
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parler_protocol::{EndpointRef, Part};

    fn message(parts: Vec<Part>) -> StoredMessage {
        StoredMessage {
            seq: 1,
            id: "m".into(),
            room: "team".into(),
            from: EndpointRef { id: "peer".into(), name: "peer".into(), role: None },
            parts,
            mentions: None,
            reply_to: None,
            ts: 1,
        }
    }

    #[test]
    fn attention_only_wakes_the_traffic_each_mode_admits() {
        let ambient = message(vec![Part::text("hello")]);
        let handoff = message(vec![
            HandoffRef { next: "ship it".into(), summary: None, to: Some("reviewer".into()), bundle: None }.to_part(),
        ]);
        let dispatch = message(vec![DispatchRef { role: "reviewer".into() }.to_part()]);
        let dnd = AttentionPolicy { mode: Attention::Dnd, ..Default::default() };
        let focus = AttentionPolicy { mode: Attention::Focus, ..Default::default() };

        assert_eq!(dnd.decide("team", RoomKind::Channel, &ambient, "bob", Some("reviewer")), AttentionDecision::Hold);
        assert_eq!(dnd.decide("team", RoomKind::Dm, &ambient, "bob", Some("reviewer")), AttentionDecision::Wake);
        assert_eq!(focus.decide("team", RoomKind::Dm, &ambient, "bob", Some("reviewer")), AttentionDecision::Hold);
        assert_eq!(focus.decide("team", RoomKind::Channel, &handoff, "bob", Some("reviewer")), AttentionDecision::Wake);
        assert_eq!(focus.decide("team", RoomKind::Service, &dispatch, "bob", Some("reviewer")), AttentionDecision::Wake);
    }

    #[test]
    fn quiet_holds_ambient_and_muted_drops_everything() {
        let msg = message(vec![Part::text("ambient")]);
        let mut policy = AttentionPolicy::default();
        policy.set_room("team", RoomAttention::Quiet);
        assert_eq!(policy.decide("team", RoomKind::Channel, &msg, "bob", None), AttentionDecision::Hold);
        policy.set_room("team", RoomAttention::Muted);
        assert_eq!(policy.decide("team", RoomKind::Channel, &msg, "bob", None), AttentionDecision::Drop);
        policy.set_room("team", RoomAttention::Inherit);
        assert_eq!(policy.decide("team", RoomKind::Channel, &msg, "bob", None), AttentionDecision::Wake);
    }
}
