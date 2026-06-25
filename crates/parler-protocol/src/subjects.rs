//! Subject naming — the routing half of the wire contract (v0). Port of Cotal `subjects.ts`.
//!
//! ```text
//!   parler.<space>.chat.<sender>.<channel…>   multicast to a channel (dotted + hierarchical)
//!   parler.<space>.svc.<service>.<sender>     anycast to any one instance of a service
//!   parler.<space>.inst.<instance>.<sender>   unicast to one specific instance
//!   parler.<space>.ctl.<service>.<sender>     control request/reply to a service
//! ```
//!
//! [`parse_subject`] is the **single authority** on the subject layout — the sender-position
//! asymmetry (idx 3 for chat, idx 4 for the rest) lives in exactly one place.

use crate::ROOT;
use std::collections::BTreeSet;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SubjectError {
    #[error("channel \"{0}\": '>' is only valid as the last segment")]
    WildcardNotLast(String),
    #[error("invalid channel \"{channel}\": {reason}")]
    InvalidChannel { channel: String, reason: String },
}

/// Make a string safe to use as a single NATS subject token: keep `[A-Za-z0-9_-]`, map the rest to
/// `_`, and never return empty.
pub fn token(s: &str) -> String {
    let t: String = s
        .trim()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if t.is_empty() {
        "_".to_string()
    } else {
        t
    }
}

pub fn space_prefix(space: &str) -> String {
    format!("{ROOT}.{}", token(space))
}

/// Canonicalize a `mentions` list for the wire: trim, lowercase, drop empties, dedupe. Returns
/// `None` for an empty result so the field is omitted rather than sent as `[]`.
pub fn normalize_mentions(mentions: &[String]) -> Option<Vec<String>> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for m in mentions {
        let m = m.trim().to_lowercase();
        if !m.is_empty() && seen.insert(m.clone()) {
            out.push(m);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Build the channel portion of a chat subject, preserving NATS hierarchy and whole-segment
/// wildcards (`*` one level, `>` the rest; `>` last only). `team.backend` → `team.backend`.
pub fn channel_path(channel: &str) -> Result<String, SubjectError> {
    let segs: Vec<&str> = channel
        .split('.')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if segs.is_empty() {
        return Ok("_".to_string());
    }
    let mut out = Vec::with_capacity(segs.len());
    for (i, s) in segs.iter().enumerate() {
        if *s == ">" {
            if i != segs.len() - 1 {
                return Err(SubjectError::WildcardNotLast(channel.to_string()));
            }
            out.push(">".to_string());
        } else if *s == "*" {
            out.push("*".to_string());
        } else {
            out.push(token(s));
        }
    }
    Ok(out.join("."))
}

/// A routing token (sender, target, role, service): preserve the literal `*` wildcard, sanitize
/// everything else. A no-op on real nkey ids.
pub fn route_token(s: &str) -> String {
    if s == "*" {
        "*".to_string()
    } else {
        token(s)
    }
}

pub fn chat_subject(space: &str, sender: &str, channel: &str) -> Result<String, SubjectError> {
    Ok(format!(
        "{}.chat.{}.{}",
        space_prefix(space),
        route_token(sender),
        channel_path(channel)?
    ))
}

/// True if a channel names a concrete sub-channel (no `*`/`>`) — i.e. it can be *published* to.
pub fn is_concrete_channel(channel: &str) -> bool {
    !channel
        .split('.')
        .any(|s| s.trim() == "*" || s.trim() == ">")
}

/// Does NATS subject `pattern` (with `*`/`>`) match `subject`? Also reused for channel-level
/// matching, since channels are dotted token strings with the same rules.
pub fn subject_matches(pattern: &str, subject: &str) -> bool {
    let p: Vec<&str> = pattern.split('.').collect();
    let s: Vec<&str> = subject.split('.').collect();
    for (i, pi) in p.iter().enumerate() {
        if *pi == ">" {
            // '>' matches one-or-more remaining tokens (NATS: 'a.>' does NOT match bare 'a').
            return i < s.len();
        }
        if i >= s.len() {
            return false;
        }
        if *pi == "*" {
            continue;
        }
        if *pi != s[i] {
            return false;
        }
    }
    p.len() == s.len()
}

/// Validate a channel name/pattern used as **policy** (`allowSubscribe`/`allowPublish`, a CLI flag,
/// a join target). Each dotted segment must be a NATS-safe token, or `*`, or `>` (final only).
/// Rejects — fail-loud — anything [`token`] would silently rewrite. Returns the channel on success.
pub fn assert_valid_channel(channel: &str) -> Result<&str, SubjectError> {
    let segs: Vec<&str> = channel.split('.').collect();
    if channel.is_empty() || segs.iter().any(|s| s.is_empty()) {
        return Err(SubjectError::InvalidChannel {
            channel: channel.to_string(),
            reason: "empty segment (no leading/trailing/double dots)".into(),
        });
    }
    for (i, s) in segs.iter().enumerate() {
        if *s == ">" {
            if i != segs.len() - 1 {
                return Err(SubjectError::WildcardNotLast(channel.to_string()));
            }
            continue;
        }
        if *s == "*" {
            continue;
        }
        if !s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-') {
            return Err(SubjectError::InvalidChannel {
                channel: channel.to_string(),
                reason: format!(
                    "segment \"{s}\" must be a NATS-safe token ([A-Za-z0-9_-]), '*', or '>'"
                ),
            });
        }
    }
    Ok(channel)
}

/// Is `channel` within a read/post ACL `allow` (a list of channel patterns)? True when some entry
/// covers it — exact, or a wildcard subtree (`team.>` covers `team.backend`).
pub fn channel_in_allow(allow: &[String], channel: &str) -> bool {
    allow.iter().any(|a| subject_matches(a, channel))
}

/// Drop exact duplicates and any subject subsumed by a more-general one — JetStream rejects a
/// consumer whose `filter_subjects` overlap, so `[team.>, team.backend]` must collapse to `[team.>]`.
pub fn collapse_filter_subjects(subjects: &[String]) -> Vec<String> {
    let uniq: Vec<String> = {
        let mut seen = BTreeSet::new();
        subjects
            .iter()
            .filter(|s| seen.insert((*s).clone()))
            .cloned()
            .collect()
    };
    uniq.iter()
        .filter(|x| !uniq.iter().any(|y| y != *x && subject_matches(y, x)))
        .cloned()
        .collect()
}

/// Unicast: a specific instance's inbox, tagged with the sender.
pub fn unicast_subject(space: &str, target: &str, sender: &str) -> String {
    format!(
        "{}.inst.{}.{}",
        space_prefix(space),
        route_token(target),
        route_token(sender)
    )
}

/// Anycast: a service (role), tagged with the sender. Subscribers join a queue group.
pub fn anycast_subject(space: &str, service: &str, sender: &str) -> String {
    format!(
        "{}.svc.{}.{}",
        space_prefix(space),
        route_token(service),
        route_token(sender)
    )
}

/// Control request/reply to a service (e.g. the manager), tagged with the sender.
pub fn control_service_subject(space: &str, service: &str, sender: &str) -> String {
    format!(
        "{}.ctl.{}.{}",
        space_prefix(space),
        route_token(service),
        route_token(sender)
    )
}

// ---- Control-plane service names — the three-tier split + the delivery daemon. ----
pub const CONTROL_PRIVILEGED: &str = "manager";
pub const CONTROL_SELF_SERVICE: &str = "self";
pub const CONTROL_ADMIN: &str = "admin";
pub const CONTROL_DELIVERY: &str = "delivery";

pub fn trace_subject(space: &str, agent_id: &str) -> String {
    format!("{}.trace.{}", space_prefix(space), token(agent_id))
}

pub fn control_subject(space: &str, agent_id: &str) -> String {
    format!("{}.control.{}", space_prefix(space), token(agent_id))
}

/// Wildcard matching every subject within a space.
pub fn space_wildcard(space: &str) -> String {
    format!("{}.>", space_prefix(space))
}

/// Wildcard matching every chat (multicast) subject in a space — the observer read surface.
pub fn chat_wildcard(space: &str) -> String {
    format!("{}.chat.>", space_prefix(space))
}

/// The three peer-message delivery modes (control/trace/presence are not deliveries).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryMode {
    Chat,
    Anycast,
    Unicast,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubjectKind {
    Chat,
    Inst,
    Svc,
    Ctl,
}

impl SubjectKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubjectKind::Chat => "chat",
            SubjectKind::Inst => "inst",
            SubjectKind::Svc => "svc",
            SubjectKind::Ctl => "ctl",
        }
    }
}

/// A subject parsed into its routing parts. `sender` is the publishing agent's id; `rest` is the
/// channel (chat) or the routed target/role/service (inst/svc/ctl).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSubject {
    pub kind: SubjectKind,
    pub sender: String,
    pub rest: String,
}

/// The single authority on the subject layout. Returns `None` on anything malformed, so a bad
/// subject can never be read as if it carried a sender.
pub fn parse_subject(subject: &str) -> Option<ParsedSubject> {
    let parts: Vec<&str> = subject.split('.').collect();
    if parts.first().copied() != Some(ROOT) {
        return None;
    }
    match parts.get(2).copied() {
        Some("chat") => {
            if parts.len() < 5 {
                return None;
            }
            Some(ParsedSubject {
                kind: SubjectKind::Chat,
                sender: parts[3].to_string(),
                rest: parts[4..].join("."),
            })
        }
        Some(k @ ("inst" | "svc" | "ctl")) => {
            if parts.len() != 5 {
                return None;
            }
            let kind = match k {
                "inst" => SubjectKind::Inst,
                "svc" => SubjectKind::Svc,
                _ => SubjectKind::Ctl,
            };
            Some(ParsedSubject {
                kind,
                sender: parts[4].to_string(),
                rest: parts[3].to_string(),
            })
        }
        _ => None,
    }
}

/// Classify a subject's delivery mode, or `None` for control/trace/etc.
pub fn delivery_of(subject: &str) -> Option<DeliveryMode> {
    let p = parse_subject(subject)?;
    match p.kind {
        SubjectKind::Chat => Some(DeliveryMode::Chat),
        SubjectKind::Svc => Some(DeliveryMode::Anycast),
        SubjectKind::Inst => Some(DeliveryMode::Unicast),
        SubjectKind::Ctl => None,
    }
}

// ---- KV bucket names (per space) ----
pub fn presence_bucket(space: &str) -> String {
    format!("parler_presence_{}", token(space))
}
pub fn channel_bucket(space: &str) -> String {
    format!("parler_channels_{}", token(space))
}
pub fn members_bucket(space: &str) -> String {
    format!("parler_members_{}", token(space))
}
pub fn acl_bucket(space: &str) -> String {
    format!("parler_acl_{}", token(space))
}
pub fn membership_bucket(space: &str) -> String {
    format!("parler_membership_{}", token(space))
}
pub fn delivery_bucket(space: &str) -> String {
    format!("parler_delivery_{}", token(space))
}

/// Reserved registry key for the space-wide channel defaults (`=` is a char `token()` never emits).
pub const CHANNEL_DEFAULTS_KEY: &str = "=defaults";
/// Reserved membership-bucket key for the feed's freshness heartbeat.
pub const MEMBERSHIP_FEED_KEY: &str = "=feed";
/// The scoped client inbox prefix the membership observer connects under.
pub const MEMBERSHIP_INBOX_PREFIX: &str = "_INBOX.parler-membership";

/// KV key for one membership record: `<channel>/<owner>`.
pub fn member_key(channel: &str, owner: &str) -> String {
    format!("{channel}/{owner}")
}

/// Inverse of [`member_key`], or `None` if it isn't one (no single internal `/`).
pub fn parse_member_key(key: &str) -> Option<(String, String)> {
    let i = key.find('/')?;
    if i == 0 || i >= key.len() - 1 {
        return None;
    }
    Some((key[..i].to_string(), key[i + 1..].to_string()))
}

pub fn acl_key(owner: &str) -> String {
    token(owner)
}
pub fn membership_key(id: &str) -> String {
    token(id)
}
pub fn lease_key(shard_index: u32) -> String {
    format!("lease.{shard_index}")
}

/// Deterministic FNV-1a (32-bit) hash of `key` into `[0, n)`. N=1 is the only shipped mode.
pub fn partition(n: u32, key: &str) -> u32 {
    if n <= 1 {
        return 0;
    }
    let mut h: u32 = 0x811c_9dc5;
    for c in key.encode_utf16() {
        h ^= c as u32;
        h = h.wrapping_mul(0x0100_0193);
    }
    h % n
}

// ---- JetStream stream names (the durable backing for the delivery modes) ----
pub fn chat_stream(space: &str) -> String {
    format!("CHAT_{}", token(space))
}
pub fn dm_stream(space: &str) -> String {
    format!("DM_{}", token(space))
}
pub fn task_stream(space: &str) -> String {
    format!("TASK_{}", token(space))
}
pub fn inbox_stream(space: &str) -> String {
    format!("INBOX_{}", token(space))
}
pub fn dlv_stream(space: &str) -> String {
    format!("DLV_{}", token(space))
}

/// Subject of an owner's mixed durable inbox: `parler.<space>.dinbox.<owner>`.
pub fn dinbox_subject(space: &str, owner: &str) -> String {
    format!("{}.dinbox.{}", space_prefix(space), route_token(owner))
}
/// Subject of an owner's post-auth delivery: `parler.<space>.dlv.<owner>`.
pub fn dlv_subject(space: &str, owner: &str) -> String {
    format!("{}.dlv.{}", space_prefix(space), route_token(owner))
}

/// Parse the owner id out of an owner's mixed-inbox subject, or `None`.
pub fn parse_dinbox_owner(subject: &str) -> Option<String> {
    let parts: Vec<&str> = subject.split('.').collect();
    if parts.len() == 4 && parts[0] == ROOT && parts[2] == "dinbox" {
        Some(parts[3].to_string())
    } else {
        None
    }
}

pub fn dlv_durable(owner: &str) -> String {
    format!("dlv_{}", token(owner))
}

/// The single privileged fan-out consumer on the CHAT stream (N=1 keeps this exact name).
pub const FANOUT_DURABLE: &str = "fanout";
/// The single privileged trusted-reader consumer on the INBOX stream (N=1 keeps this exact name).
pub const INBOX_READER_DURABLE: &str = "reader";

pub fn fanout_durable(shard: u32, shards: u32) -> String {
    if shards <= 1 {
        FANOUT_DURABLE.to_string()
    } else {
        format!("{FANOUT_DURABLE}_{shard}")
    }
}
pub fn reader_durable(shard: u32, shards: u32) -> String {
    if shards <= 1 {
        INBOX_READER_DURABLE.to_string()
    } else {
        format!("{INBOX_READER_DURABLE}_{shard}")
    }
}

/// Name of the REMOVED per-instance chat live-tail durable (retained as the name an agent must NOT
/// be able to create).
pub fn chat_durable(instance: &str) -> String {
    format!("chat_{}", token(instance))
}
/// Consumer name for an instance's short-lived chat **history** reads (join-backfill, focus-recall).
pub fn chat_hist_durable(instance: &str) -> String {
    format!("chathist_{}", token(instance))
}
/// Durable consumer name for an instance's private DM inbox.
pub fn dm_durable(instance: &str) -> String {
    format!("dm_{}", token(instance))
}
/// Durable consumer name (shared across instances of a role) for the task queue.
pub fn task_durable(service: &str) -> String {
    format!("svc_{}", token(service))
}

/// Extract the channel pattern from a live chat SUBSCRIPTION subject in this space, or `None`.
pub fn channel_from_chat_subscription(space: &str, subject: &str) -> Option<String> {
    if !subject.starts_with(&format!("{}.chat.", space_prefix(space))) {
        return None;
    }
    parse_subject(subject).and_then(|p| {
        if p.kind == SubjectKind::Chat {
            Some(p.rest)
        } else {
            None
        }
    })
}

// ---- $SYS account subjects (membership observer; CONNZ-derived feed) ----
pub fn connz_request_subject(account_id: &str) -> String {
    format!("$SYS.REQ.ACCOUNT.{account_id}.CONNZ")
}
pub fn account_connect_subject(account_id: &str) -> String {
    format!("$SYS.ACCOUNT.{account_id}.CONNECT")
}
pub fn account_disconnect_subject(account_id: &str) -> String {
    format!("$SYS.ACCOUNT.{account_id}.DISCONNECT")
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALICE: &str = "UAQGWOEVJKMIO4WXSYOTLARXYOZTCXFK67JASEH6AFFFYK6FOPSKQCAD";
    const BOB: &str = "UDI36ZKVNUM5WMO4QQ6HDQU7F4OH2RCXOJRX6GAIOS5SKVNNSKCDNLJA";

    /// SPEC §12 subject-parse vectors, rebranded to the `parler` root.
    #[test]
    fn spec_subject_parse_vectors() {
        let chat = parse_subject(&format!("parler.main.chat.{ALICE}.team.backend")).unwrap();
        assert_eq!(chat.kind, SubjectKind::Chat);
        assert_eq!(chat.sender, ALICE);
        assert_eq!(chat.rest, "team.backend");

        let inst = parse_subject(&format!("parler.main.inst.{BOB}.{ALICE}")).unwrap();
        assert_eq!(inst.kind, SubjectKind::Inst);
        assert_eq!(inst.sender, ALICE);
        assert_eq!(inst.rest, BOB);

        let svc = parse_subject(&format!("parler.main.svc.reviewer.{ALICE}")).unwrap();
        assert_eq!(svc.kind, SubjectKind::Svc);
        assert_eq!(svc.sender, ALICE);
        assert_eq!(svc.rest, "reviewer");

        let ctl = parse_subject(&format!("parler.main.ctl.manager.{ALICE}")).unwrap();
        assert_eq!(ctl.kind, SubjectKind::Ctl);
        assert_eq!(ctl.sender, ALICE);
        assert_eq!(ctl.rest, "manager");

        // No sender ⇒ malformed chat subject.
        assert!(parse_subject(&format!("parler.main.chat.{ALICE}")).is_none());
        // Wrong root.
        assert!(parse_subject(&format!("cotal.main.svc.reviewer.{ALICE}")).is_none());
    }

    #[test]
    fn delivery_of_classifies_by_subject_not_payload() {
        assert_eq!(
            delivery_of(&format!("parler.main.chat.{ALICE}.general")),
            Some(DeliveryMode::Chat)
        );
        assert_eq!(
            delivery_of(&format!("parler.main.inst.{BOB}.{ALICE}")),
            Some(DeliveryMode::Unicast)
        );
        assert_eq!(
            delivery_of(&format!("parler.main.svc.reviewer.{ALICE}")),
            Some(DeliveryMode::Anycast)
        );
        assert_eq!(
            delivery_of(&format!("parler.main.ctl.manager.{ALICE}")),
            None
        );
    }

    #[test]
    fn subject_matching_follows_nats_semantics() {
        assert!(subject_matches("team.>", "team.backend"));
        assert!(subject_matches("team.>", "team.backend.api"));
        assert!(!subject_matches("team.>", "team")); // '>' needs ≥1 more token
        assert!(subject_matches("*", "team"));
        assert!(!subject_matches("*", "team.backend"));
        assert!(subject_matches("review", "review"));
        assert!(!subject_matches("review.>", "review")); // disjoint in NATS
    }

    #[test]
    fn collapse_drops_subsumed_keeps_disjoint() {
        assert_eq!(
            collapse_filter_subjects(&["team.>".into(), "team.backend".into()]),
            vec!["team.>".to_string()]
        );
        // A parent and its subtree are disjoint in NATS — both kept.
        assert_eq!(
            collapse_filter_subjects(&["review".into(), "review.>".into()]),
            vec!["review".to_string(), "review.>".to_string()]
        );
        assert_eq!(
            collapse_filter_subjects(&["a".into(), "a".into()]),
            vec!["a".to_string()]
        );
    }

    #[test]
    fn channel_validation_and_concreteness() {
        assert!(assert_valid_channel("team.backend").is_ok());
        assert!(assert_valid_channel("team.>").is_ok());
        assert!(assert_valid_channel("*").is_ok());
        assert!(matches!(
            assert_valid_channel("team..backend"),
            Err(SubjectError::InvalidChannel { .. })
        ));
        assert!(matches!(
            assert_valid_channel("team.>.x"),
            Err(SubjectError::WildcardNotLast(_))
        ));
        assert!(matches!(
            assert_valid_channel("foo/bar"),
            Err(SubjectError::InvalidChannel { .. })
        ));
        assert!(is_concrete_channel("team.backend"));
        assert!(!is_concrete_channel("team.>"));
        assert!(!is_concrete_channel("a.*.b"));
    }

    #[test]
    fn channel_in_allow_covers_subtrees() {
        assert!(channel_in_allow(&["team.>".into()], "team.backend"));
        assert!(channel_in_allow(&["review".into()], "review"));
        assert!(!channel_in_allow(&["review".into()], "team"));
    }

    #[test]
    fn mentions_normalized() {
        assert_eq!(
            normalize_mentions(&["Bob".into(), " alice ".into(), "".into(), "bob".into()]),
            Some(vec!["bob".to_string(), "alice".to_string()])
        );
        assert_eq!(normalize_mentions(&[]), None);
        assert_eq!(normalize_mentions(&["".into(), "  ".into()]), None);
    }

    #[test]
    fn member_key_round_trips() {
        assert_eq!(member_key("team.backend", "UABC"), "team.backend/UABC");
        assert_eq!(
            parse_member_key("team.backend/UABC"),
            Some(("team.backend".to_string(), "UABC".to_string()))
        );
        assert_eq!(parse_member_key("nope"), None);
        assert_eq!(parse_member_key("/x"), None);
        assert_eq!(parse_member_key("x/"), None);
    }

    #[test]
    fn partition_is_deterministic_and_n1_is_zero() {
        assert_eq!(partition(1, "anything"), 0);
        assert_eq!(partition(4, "abc"), partition(4, "abc"));
        assert!(partition(4, "abc") < 4);
    }

    #[test]
    fn subject_and_name_builders_use_parler_root() {
        assert_eq!(
            chat_subject("main", ALICE, "team.backend").unwrap(),
            format!("parler.main.chat.{ALICE}.team.backend")
        );
        assert_eq!(
            unicast_subject("main", BOB, ALICE),
            format!("parler.main.inst.{BOB}.{ALICE}")
        );
        assert_eq!(presence_bucket("main"), "parler_presence_main");
        assert_eq!(chat_stream("main"), "CHAT_main");
        assert_eq!(parse_dinbox_owner("parler.main.dinbox.UABC"), Some("UABC".into()));
        assert_eq!(
            channel_from_chat_subscription("main", &format!("parler.main.chat.*.team.>")),
            Some("team.>".to_string())
        );
    }

    #[test]
    fn token_sanitizes() {
        assert_eq!(token("hello world"), "hello_world");
        assert_eq!(token("a.b/c"), "a_b_c");
        assert_eq!(token("  "), "_");
        assert_eq!(token("Keep-9_x"), "Keep-9_x");
    }
}
