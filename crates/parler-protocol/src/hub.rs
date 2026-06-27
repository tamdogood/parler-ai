//! parler-protocol::hub — wire frames for the Parler **Hub**, the lightweight "Slack for agents"
//! transport (a WebSocket bus + an embedded durable store).
//!
//! These are the frames an agent's client (`parler-connector`) exchanges with the hub
//! (`parler-hub`). Like the rest of this crate they are pure serde types that perform no IO, so the
//! client and the server share one definition of the protocol. JSON field names that are multi-word
//! are camelCase to match the rest of the Parler envelope.

use crate::types::{AgentCard, EndpointRef, Part};
use serde::{Deserialize, Serialize};

/// What kind of room a name refers to. The three delivery patterns are all just rooms with
/// different membership shapes: a [`RoomKind::Channel`] with N members is one-to-many, a two-member
/// [`RoomKind::Dm`] is one-to-one, and a [`RoomKind::Service`] room that publishers share with the
/// worker(s) is many-to-one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoomKind {
    Channel,
    Dm,
    Service,
}

impl RoomKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoomKind::Channel => "channel",
            RoomKind::Dm => "dm",
            RoomKind::Service => "service",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "channel" => Some(RoomKind::Channel),
            "dm" => Some(RoomKind::Dm),
            "service" => Some(RoomKind::Service),
            _ => None,
        }
    }
}

/// Who may discover an agent in the hub's directory. **Secure by default:** an agent is
/// [`Visibility::Private`] unless it explicitly opts into [`Visibility::Public`].
///
/// - [`Visibility::Public`] — listed in the hub's world-readable public directory; discoverable by
///   any agent (and by anyone hitting the public REST API).
/// - [`Visibility::Private`] — discoverable only by agents in the **same hub** (an authenticated
///   member, or a holder of a read-scoped directory token).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    Public,
    #[default]
    Private,
}

impl Visibility {
    pub fn as_str(&self) -> &'static str {
        match self {
            Visibility::Public => "public",
            Visibility::Private => "private",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "public" => Some(Visibility::Public),
            "private" => Some(Visibility::Private),
            _ => None,
        }
    }
}

/// The scope of a [`ClientFrame::Discover`] query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscoverScope {
    /// Every agent in this hub (public + private) — the "same private hub" view. Available to any
    /// authenticated member.
    #[default]
    Hub,
    /// Only this hub's `public` agents — the world-readable directory.
    Public,
}

/// A directory record as the hub returns it on [`ServerFrame::Directory`] / [`ServerFrame::Card`].
///
/// The `card` is the agent's self-described [`AgentCard`]; `sig` is the agent's detached nkey
/// signature over [`canonical_card_bytes`] of that card. Because an agent's `id` *is* its public
/// key, any consumer can verify `sig` and know the hub did not forge or alter the card — `verified`
/// is the hub's own check of exactly that.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DirectoryEntry {
    pub card: AgentCard,
    pub visibility: Visibility,
    /// Last-known presence status (`idle`/`working`/`waiting`/`offline`).
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    /// The hub (workspace) this agent registered in.
    pub hub: String,
    /// Whether the hub verified `sig` against `card.id` at registration.
    pub verified: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sig: Option<String>,
    #[serde(rename = "firstSeen")]
    pub first_seen: i64,
    #[serde(rename = "lastSeen")]
    pub last_seen: i64,
}

/// Where a [`ClientFrame::Send`] is addressed. The hub resolves each to the concrete room it stores
/// the message under, so the three patterns share one code path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Target {
    /// One-to-many (or many-to-one): a named channel room.
    Room { room: String },
    /// One-to-one: the DM room shared with `agent` (established by a prior `dm` invite + redeem).
    Dm { agent: String },
    /// Many-to-one: a service room (`svc.<service>`) shared by requesters and the worker(s).
    Service { service: String },
}

/// A fact in the memory store. `key` makes a write idempotent — the same key overwrites within its
/// scope rather than appending a duplicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fact {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    pub text: String,
    /// Room scope; `None` = the author's private memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
}

/// A stored message as the hub returns it on [`ClientFrame::Pull`] (room-scoped, with its monotonic
/// per-hub `seq` — the cursor unit).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoredMessage {
    pub seq: i64,
    pub id: String,
    pub room: String,
    pub from: EndpointRef,
    pub parts: Vec<Part>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mentions: Option<Vec<String>>,
    #[serde(default, rename = "replyTo", skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
    pub ts: i64,
}

/// A recall hit: the matched fact plus its relevance (BM25 — lower is a better match).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecallHit {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub room: Option<String>,
    pub author: String,
    pub ts: i64,
    pub score: f64,
}

/// A roster entry — a room member with their last-known presence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RosterEntry {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity: Option<String>,
    #[serde(rename = "lastSeen")]
    pub last_seen: i64,
}

/// A room the calling agent belongs to, with its unread count (messages past the agent's cursor).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomInfo {
    pub name: String,
    pub kind: RoomKind,
    pub members: u32,
    pub unread: u32,
}

/// Frames the client sends to the hub.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ClientFrame {
    /// Identify + authenticate. Sent first with `sig = None`; the hub replies
    /// [`ServerFrame::Challenge`] and the client re-sends with `nonce` echoed and `sig` = the
    /// base64 nkey signature over that nonce (proves it owns the `id` keypair).
    Hello {
        id: String,
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        role: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        nonce: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },
    /// Mint an invite code/link. `room` is optional for `channel` (auto-named when absent) and
    /// ignored for `dm` (a fresh DM room is always created).
    Invite {
        kind: RoomKind,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        room: Option<String>,
        #[serde(default, rename = "ttlSecs", skip_serializing_if = "Option::is_none")]
        ttl_secs: Option<u64>,
        #[serde(default, rename = "maxUses", skip_serializing_if = "Option::is_none")]
        max_uses: Option<u32>,
    },
    /// Redeem a pasted code — joins the room it grants.
    Redeem { code: String },
    /// Join/create a service room as a worker, so `Send`/`Pull` on it are authorized.
    Serve { service: String },
    /// Publish (or refresh) this agent's directory card. `card.id` must equal the authenticated
    /// agent id; `sig` is the agent's nkey signature over [`canonical_card_bytes`] of `card`, which
    /// the hub verifies so the stored entry is tamper-evident.
    Register {
        card: AgentCard,
        #[serde(default)]
        visibility: Visibility,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sig: Option<String>,
    },
    /// Search the directory. [`DiscoverScope::Hub`] returns every agent in this hub;
    /// [`DiscoverScope::Public`] returns only its `public` agents. Optional `query`/`tag`/`skill`/
    /// `status` narrow the result.
    Discover {
        #[serde(default)]
        scope: DiscoverScope,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        query: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        tag: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skill: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        status: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
    /// Fetch a single agent's directory card by id.
    Lookup { id: String },
    /// Mint a read-scoped, expiring directory token (a bearer the website pastes to view this hub's
    /// private directory over the REST API).
    MintDirectoryToken {
        #[serde(default, rename = "ttlSecs", skip_serializing_if = "Option::is_none")]
        ttl_secs: Option<u64>,
    },
    /// Publish a message to a target.
    Send {
        target: Target,
        parts: Vec<Part>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mentions: Option<Vec<String>>,
        #[serde(default, rename = "replyTo", skip_serializing_if = "Option::is_none")]
        reply_to: Option<String>,
    },
    /// Pull messages for a room newer than the agent's stored cursor (which this advances), or newer
    /// than `since` (which does not advance the cursor — for re-reads).
    Pull {
        room: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        since: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
    /// Write a fact to the memory store.
    Remember { fact: Fact },
    /// Full-text recall from the memory store (scoped to `room` if given, else the agent's reachable
    /// memory: its private facts plus the rooms it belongs to).
    Recall {
        query: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        room: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
    /// List the rooms the agent belongs to.
    Rooms,
    /// The members + presence of a room.
    Roster { room: String },
    /// Advertise presence.
    Presence {
        status: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        activity: Option<String>,
    },
    Ping,
}

/// Frames the hub sends back. Every non-error frame is the direct reply to the client's last op.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerFrame {
    /// Step 1 of the handshake: sign this `nonce` and re-send `Hello`.
    Challenge { nonce: String },
    /// Handshake complete — the connection is authenticated as `id`.
    Welcome { id: String, name: String },
    Invited {
        code: String,
        url: String,
        room: String,
        kind: RoomKind,
        #[serde(rename = "expiresAt")]
        expires_at: i64,
    },
    Joined {
        room: String,
        kind: RoomKind,
    },
    Registered {
        id: String,
        visibility: Visibility,
        verified: bool,
    },
    Directory {
        agents: Vec<DirectoryEntry>,
    },
    Card {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entry: Option<DirectoryEntry>,
    },
    DirectoryToken {
        token: String,
        #[serde(rename = "expiresAt")]
        expires_at: i64,
    },
    Sent {
        id: String,
        seq: i64,
        room: String,
    },
    Pulled {
        room: String,
        messages: Vec<StoredMessage>,
        cursor: i64,
    },
    Remembered {
        ok: bool,
    },
    Recalled {
        hits: Vec<RecallHit>,
    },
    Rooms {
        rooms: Vec<RoomInfo>,
    },
    Roster {
        room: String,
        entries: Vec<RosterEntry>,
    },
    PresenceOk,
    Pong,
    Error {
        message: String,
    },
}

/// The canonical byte encoding of an [`AgentCard`] for signing/verification.
///
/// Produces a deterministic, whitespace-free JSON with **recursively key-sorted** objects (an
/// RFC 8785 / JCS-style canonical form, robust even if `serde_json` is built with `preserve_order`).
/// Both the signer (the agent) and the verifier (the hub, or any client) feed these exact bytes to
/// the nkey sign/verify so a card cannot be silently altered after signing.
pub fn canonical_card_bytes(card: &AgentCard) -> Vec<u8> {
    let v = serde_json::to_value(card).unwrap_or(serde_json::Value::Null);
    serde_json::to_vec(&canonicalize(&v)).unwrap_or_default()
}

/// Recursively rebuild a JSON value with object keys in sorted order.
fn canonicalize(v: &serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::Object(m) => {
            let mut keys: Vec<&String> = m.keys().collect();
            keys.sort();
            let mut sorted = serde_json::Map::with_capacity(m.len());
            for k in keys {
                sorted.insert(k.clone(), canonicalize(&m[k]));
            }
            Value::Object(sorted)
        }
        Value::Array(a) => Value::Array(a.iter().map(canonicalize).collect()),
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{AgentSkill, EndpointKind};
    use std::collections::BTreeMap;

    fn sample_card() -> AgentCard {
        let mut meta = BTreeMap::new();
        meta.insert("zone".to_string(), serde_json::json!("us-east"));
        meta.insert("region".to_string(), serde_json::json!("global"));
        AgentCard {
            id: "UABC".into(),
            name: "alice".into(),
            kind: EndpointKind::Agent,
            role: Some("planner".into()),
            description: Some("plans things".into()),
            tags: Some(vec!["planning".into(), "ops".into()]),
            skills: Some(vec![AgentSkill {
                id: "plan".into(),
                name: "Planning".into(),
                description: None,
            }]),
            meta: Some(meta),
            protocol_version: Some("0.2".into()),
        }
    }

    #[test]
    fn canonical_card_bytes_is_deterministic_and_key_sorted() {
        let card = sample_card();
        let a = canonical_card_bytes(&card);
        let b = canonical_card_bytes(&card.clone());
        assert_eq!(a, b, "canonicalization must be stable");
        let s = String::from_utf8(a).unwrap();
        // Object keys are sorted: `description` precedes `id` precedes `name`; nested meta `region`
        // precedes `zone`. And there is no insignificant whitespace.
        assert!(s.find("\"description\"").unwrap() < s.find("\"id\"").unwrap());
        assert!(s.find("\"region\"").unwrap() < s.find("\"zone\"").unwrap());
        assert!(!s.contains(": "));
    }

    #[test]
    fn directory_entry_round_trips_camelcase() {
        let entry = DirectoryEntry {
            card: sample_card(),
            visibility: Visibility::Public,
            status: "working".into(),
            activity: Some("planning the sprint".into()),
            hub: "Parler Public".into(),
            verified: true,
            sig: Some("AAAA".into()),
            first_seen: 10,
            last_seen: 20,
        };
        let j = serde_json::to_value(&entry).unwrap();
        assert_eq!(j["visibility"], "public");
        assert_eq!(j["verified"], true);
        assert_eq!(j["firstSeen"], 10);
        assert_eq!(j["lastSeen"], 20);
        assert_eq!(j["card"]["id"], "UABC");
        let back: DirectoryEntry = serde_json::from_value(j).unwrap();
        assert_eq!(back, entry);
    }

    #[test]
    fn discovery_frames_round_trip() {
        let reg = ClientFrame::Register {
            card: sample_card(),
            visibility: Visibility::Public,
            sig: Some("SIG".into()),
        };
        let j = serde_json::to_value(&reg).unwrap();
        assert_eq!(j["op"], "register");
        assert_eq!(j["visibility"], "public");
        assert_eq!(serde_json::from_value::<ClientFrame>(j).unwrap(), reg);

        let disc = ClientFrame::Discover {
            scope: DiscoverScope::Public,
            query: Some("plan".into()),
            tag: None,
            skill: Some("review".into()),
            status: None,
            limit: Some(20),
        };
        let j = serde_json::to_value(&disc).unwrap();
        assert_eq!(j["op"], "discover");
        assert_eq!(j["scope"], "public");
        assert_eq!(serde_json::from_value::<ClientFrame>(j).unwrap(), disc);
    }

    #[test]
    fn visibility_defaults_to_private() {
        assert_eq!(Visibility::default(), Visibility::Private);
        // An absent `visibility`/`scope` deserializes to the secure-by-default values.
        let reg: ClientFrame =
            serde_json::from_value(serde_json::json!({"op":"register","card":sample_card()}))
                .unwrap();
        match reg {
            ClientFrame::Register { visibility, .. } => assert_eq!(visibility, Visibility::Private),
            _ => panic!("expected register"),
        }
        let disc: ClientFrame =
            serde_json::from_value(serde_json::json!({"op":"discover"})).unwrap();
        match disc {
            ClientFrame::Discover { scope, .. } => assert_eq!(scope, DiscoverScope::Hub),
            _ => panic!("expected discover"),
        }
    }

    #[test]
    fn client_frame_round_trips_with_op_tag() {
        let f = ClientFrame::Send {
            target: Target::Dm {
                agent: "UABC".into(),
            },
            parts: vec![Part::text("hi")],
            mentions: None,
            reply_to: None,
        };
        let j = serde_json::to_value(&f).unwrap();
        assert_eq!(j["op"], "send");
        assert_eq!(j["target"]["kind"], "dm");
        assert_eq!(j["target"]["agent"], "UABC");
        assert_eq!(j["parts"][0]["kind"], "text");
        let back: ClientFrame = serde_json::from_value(j).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn server_frame_round_trips_with_type_tag() {
        let f = ServerFrame::Invited {
            code: "AB12CD34".into(),
            url: "parler://127.0.0.1:7070/join/AB12CD34".into(),
            room: "dm.x".into(),
            kind: RoomKind::Dm,
            expires_at: 123,
        };
        let j = serde_json::to_value(&f).unwrap();
        assert_eq!(j["type"], "invited");
        assert_eq!(j["kind"], "dm");
        assert_eq!(j["expiresAt"], 123);
        let back: ServerFrame = serde_json::from_value(j).unwrap();
        assert_eq!(back, f);
    }

    #[test]
    fn unit_variants_serialize_as_tag_only() {
        assert_eq!(serde_json::to_value(ClientFrame::Ping).unwrap()["op"], "ping");
        assert_eq!(serde_json::to_value(ClientFrame::Rooms).unwrap()["op"], "rooms");
        assert_eq!(
            serde_json::to_value(ServerFrame::PresenceOk).unwrap()["type"],
            "presence_ok"
        );
    }
}
