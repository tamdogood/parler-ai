# Auto-mint the watch code when a session opens

## Problem
Opening a session surfaces a prominent join `KEY`; the read-only web/desktop **watch code**
is a separate manual step (`parler_watch_session` / `parler session watch`), mentioned only
in passing. The host grabs the KEY, pastes it into the session viewer, and gets a 401 rendered
as "That code is invalid or expired" — indistinguishable from a genuinely stale watch code.
(Repro: `ZX6Y2QPX` is an 8-char join key; the viewer only accepts a 32-char watch token.)

## Fix (as requested): mint the watch code automatically at open — DONE
- [x] `open_session` (MCP): mints the watch token after the invite; surfaces a lean `WATCH code` line.
      Best-effort — falls back to the manual `parler_watch_session` hint if the hub can't mint.
- [x] TTL alignment: watch code minted with `Some(ttl_secs.unwrap_or(24*3600))` = same lifetime as the key.
- [x] `parler_open_session` tool description updated (KEY + WATCH code; "do NOT paste the KEY there").
- [x] CLI `SessionCmd::Open` mirrors — mints + prints the WATCH code (fallback on older hub).
- [x] `parler_watch_session` / `parler session watch` kept for re-minting.
- [x] Docs updated (`docs/team-sessions.md`).
- [x] Budget test: assertion updated to the new contract; `OPEN_RESULT_BUDGET` 800→900 (documented).
- [x] Verified: 105 cli lib tests + 39 connector e2e pass; clippy -D warnings clean on all touched crates.

## Notes
- Keep the literal `KEY: {code}@{hub}` line unchanged (a test helper parses `"KEY: "`).
- The distinction ("agents join with KEY; humans watch with WATCH code") rides on the WATCH line.
