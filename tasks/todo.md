# Fix: "join" should mean "you're in the room"

The reported failure: two agents in a room, but the host's roster shows 1 and can't find the
other. Root cause = `parler session join` joins, prints, and **exits** — so the joiner looks
present for a blink, then leaves. Secondary wart: a first-run bootstrap that can't reach the hub
still persists an identity, so a network-blocked join announces "initialized new agent …" and then
errors — a confusing half-success.

## Plan

- [ ] **#1 — `session join` stays in the room by default.** After a successful join, hold the
      connection open: set presence `online`, subscribe, and follow the room (print new messages,
      heartbeat presence) until Ctrl-C. This makes the joiner show `online` in the host's roster and
      receive messages live — matching everyone's mental model of "join."
  - [ ] Add `--once` flag to `SessionCmd::Join` for the old fire-and-exit behavior (scripts/CI).
  - [ ] Add a `follow_session` helper (presence heartbeat + subscribe/pull loop). Leave `cmd_recv`
        untouched (minimal impact).
- [ ] **#2 — Don't leave an orphaned identity when the first connect fails.** In `connect()`'s
      fresh-bootstrap branch, if the very first connect fails, roll back the just-minted
      `config.json` so the next attempt (after `parler doctor` fixes the network) starts clean.
      Existing identities are never touched.
  - [ ] Add `Config::remove()` to parler-connector (next to `save`/`exists`).

## Gate

- [ ] `CI_SKIP_WEB=1 make ci` green (clippy -D warnings is hard). No `cargo fmt`.
- [ ] Wire protocol untouched — CLI behavior + identity-persistence ordering only.

## Review

Done. All items landed; `CI_SKIP_WEB=1 make ci` exits 0 (clippy -D warnings clean, all tests pass).

- **#1** — `SessionCmd::Join` gained `--once`. Default path now calls a new `follow_session()`
  helper: sets presence `online`, subscribes, and loops (heartbeat presence every 120s < the 5-min
  `PRESENCE_STALE_MS` window; prints new messages) until Ctrl-C. So a joiner is now genuinely *in*
  the room — `online` in the host's roster, receiving live — instead of fire-and-exit. `--once`
  preserves the old scripted behavior. `cmd_recv` untouched.
- **#2** — Added `Config::remove()` (idempotent) + a unit test. `connect()`'s fresh-bootstrap
  branch now rolls the just-minted `config.json` back if the *first* connect fails, so a
  network-blocked first run (e.g. sandboxed DNS) no longer strands an identity. Existing identities
  are never touched.
- Docs: `docs/agent-mesh.md` updated to describe the stay-connected default + `--once`.
- Wire protocol untouched — CLI behavior + identity-persistence ordering only.

### Follow-ups (not done — out of scope of the reported bug)
- The MCP `parler_join_session` tool is inherently "join + return"; the live-in-room story there is
  the long-lived MCP server itself. Worth a docs pass steering agents to MCP over the CLI one-shot.
- Approval-*pending* join still returns early (re-run to check). Could auto-poll then follow.
