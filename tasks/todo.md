# Feature: Code Handoff — git-bundle artifact passing (2026-06-27, BUILT)

**User ask:** investigate [ottogin/agenthub](https://github.com/ottogin/agenthub) and borrow the good
stuff. Conclusion: Parler is the *communication* plane (Slack); agenthub is the *artifact* plane
(GitHub). The gap worth filling is that agents can pass messages/facts but **not work artifacts**.
Borrowed the **git-bundle transport** (not agenthub's commit-DAG/GitHub metaphor). Full spec:
`docs/code-handoff.md`.

Design in one line: a handoff = a content-addressed **blob** (sha256 of a git bundle, on the hub's
disk, bound to its room) + an ordinary room message carrying a `Part::Extension { kind:
"com.parler.bundle", ... }`. Bytes move over the **already-authenticated WebSocket as binary frames**
(no new HTTP channel, no new dep, no capability tokens). `send`/`recv`/cursor/wake all work unchanged.

## Phase 1 — blob handoff (MVP)
- [x] `parler-protocol::hub`: `PutBlob`/`GetBlob` (`ClientFrame`); `BlobReady`/`BlobStored`/`BlobIncoming` (`ServerFrame`)
- [x] `parler-protocol::hub`: `BUNDLE_KIND` const + `BundleRef::{to_part,from_part}` (+ round-trip test)
- [x] `parler-hub::store`: `blobs` + `blob_rooms` tables (metadata; bytes on disk); `BlobMeta`/`put_blob_meta`/`blob_meta`/`blob_readable_by` (+ test)
- [x] `parler-hub::server`: `PutBlob` (resolve `Target` + member + size → `BlobReady`) → consume one Binary frame (verify sha256+len) → persist → `BlobStored`
- [x] `parler-hub::server`: `GetBlob` (member-of-any-bound-room check → `BlobIncoming` + Binary frame)
- [x] `parler-hub`: `HubState::new` + `{blob_dir,max_blob_bytes}` + flags/env; `serve` creates the dir
- [x] `parler-connector::client`: `recv_binary` + `MeshTransport::{upload_blob,download_blob}`
- [x] `parler-connector::agent`: `push(target, bundle, meta, note)`, `fetch_blob(id)`, `BundleMeta`, `PushReceipt`
- [x] `parler-cli`: `push` (git bundle create → upload → post message), `fetch` (bytes only), `apply` (verify+fetch into `refs/parler/*`, never auto-merge)
- [x] `parler-cli`: `recv` renders a `com.parler.bundle` part (📦, full blob id in the apply hint)
- [x] `parler-cli::mcp`: `parler_push`, `parler_fetch` (NO apply)
- [x] e2e test: push → recv (sees bundle part) → fetch_blob → bytes match → non-member denied
- [x] content-address helper `parler_auth::content_id` (single source of truth for hub + connector)

## Phase 2 — defense (borrowed from agenthub)
- [x] `max_blob_bytes` enforced (default 25 MiB, `--max-blob-bytes`/env) at PutBlob + on the received frame
- [x] per-agent in-memory fixed-window rate limits (`RateLimits`: 240 sends/min, 120 blobs/hour) on `HubState`

## Phase 3 — frontier (deferred; possible scope creep)
- [ ] index latest bundle per room (tip/summary/author); `parler frontier --room R`; surface in `rooms`/website

## Review — 2026-06-27
Built Phase 1 + Phase 2. Decisions: **WS-binary** transport (no new dep/HTTP/token surface),
**single-frame** blobs, **25 MiB** cap, Phase 3 deferred.
- **Tests:** `--no-fail-fast` across touched crates green — protocol **24** (+blob frames, +BundleRef
  round-trip), hub **10** (+blob meta/room binding), connector e2e **7** (+`code_handoff_*`) & discovery
  **5**. Only failure is the pre-existing `parler-auth` `auth_live` (needs a vendored `nats-server`).
- **Live, real git:** two `parler` agents over a real hub — `push` a real git bundle → peer `recv`s
  the 📦 handoff → `apply` lands the **exact tip** in a fresh repo (both commits present) → non-member
  `fetch` denied → blobs persisted content-addressed under `<db>.blobs/`.
- **Clippy:** clean except a **pre-existing** `large_enum_variant` on `ServerFrame::Card` (DirectoryEntry),
  unrelated to this feature; new variants are tiny.
- **Additive / backward-compatible:** new frames + one extension kind; old clients render an unknown
  bundle part gracefully. Docs: `docs/code-handoff.md` (full spec, "as built"), `docs/agent-mesh.md`,
  `README.md` updated.

---

# Feature: Agent Discovery — directory + signed cards + Next.js site (2026-06-27)

**User ask:** the best discovery hub — agents register with a uuid + a public/private visibility
(public = discoverable by any agent; private = same-hub only), Slack-like, with a strong security
protocol, plus a Next.js + shadcn dark-theme website (Resend styling) to browse a hub or the public
directory. Confirmed: one hub binary in public/private mode; private-hub viewing via a short-lived
directory token; ship a runnable demo. Plan: `~/.claude/plans/recursive-hatching-hearth.md`.

### Built
- [x] **Protocol** (`parler-protocol::hub`): `Visibility{public,private}` (default private),
  `DiscoverScope{hub,public}`, `DirectoryEntry`, frames `Register/Discover/Lookup/MintDirectoryToken`
  + `Registered/Directory/Card/DirectoryToken`, and `canonical_card_bytes` (RFC-8785-style).
- [x] **Auth**: `parler_auth::{sign,verify}` (nkey Ed25519), reused by hub + connector + tests.
- [x] **Hub store**: `directory` + `directory_tokens` tables; `register_card`, `discover`
  (scope/tag/skill/status filters), `lookup_card`, token mint/validate; presence now self-reported
  and **decayed to offline by staleness** (`PRESENCE_STALE_MS`) instead of forced on disconnect.
- [x] **Hub server**: WS ops (verify signature, bind `card.id == authed id`); read-only REST
  `/api/hub`, `/api/directory`, `/api/agents/:id` with `tower-http` CORS + bearer-token gating for
  `scope=hub`; `--name`/`--public` flags + `HubMode`.
- [x] **Connector + CLI + MCP**: `MeshAgent::{register,discover,lookup,mint_directory_token}`
  (signs the card with the local seed); CLI `register/discover/card/token`; MCP `parler_register/
  parler_discover/parler_card`.
- [x] **Website** (`web/`): Next.js 15 + Tailwind v4 + shadcn-style, Resend dark theme — nav/hero,
  hub header, scope toggle, search + filters, signed agent cards with status + verified badges, a
  detail sheet, and a token-unlock dialog. Builds clean; screenshot-verified against a live hub.
- [x] **Demo + docs**: `scripts/seed-demo.sh` (public hub + 7 signed agents, 5 public/2 private),
  `docs/discovery.md`, pointer in `docs/agent-mesh.md`.
- [x] **Discovery → conversation bridge** (follow-up): a `register`ed agent is *reachable* — a peer
  can `send --to <id>` cold and the hub opens the DM room (no paste-a-code). `resolve_target` falls
  back to pairing only for agents with no directory card. Verified with a live two-agent round-trip
  (atlas DMs probe by id → probe reads + replies). Tests +2 in `discovery_e2e`.

### Review — 2026-06-27
- **Tests:** `cargo test --workspace --no-fail-fast` = **69 passed / 1 failed**; the single failure
  is the pre-existing `parler-auth` `auth_live` test (needs a vendored `nats-server`, unrelated).
  New: protocol +4 (frames/canonicalization/default), auth +1 (sign/verify), hub +3 (scope split,
  visibility/idempotent register, token expiry), connector +3 e2e (`discovery_e2e`: public-vs-hub
  visibility, forged/tampered/unsigned card handling, token mint).
- **Live demo verified:** `/api/hub` → public hub "Parler Public", 7 agents/5 public; public
  directory returns the 5 public agents (all `verified:true`); hub scope returns all 7; `parler
  discover --public` matches; the website renders the cards (headless-Chrome screenshot).
- **Security highlight:** cards are self-signed by the agent's own nkey; the hub stores + verifies
  but cannot forge them — `verified` is independently checkable by any client.

---

# Feature: Agent Mesh — "Slack for agents" (focused build)

**2026-06-27 — user redirected scope.** Not a full Cotal copy. Deliver a focused feature: any agent
(Claude Code / Codex / Hermes) talks to any other in **1:1, many:1, 1:many**; an **efficient memory
backend**; and **paste-a-code pairing** ("tell my agent → it hands me a link/code → I paste it to the
other agent → the connection persists"). Must be **fast, low-cost, low-ops**.

### Architecture (proposed — confirm before building)
- **`parler-hub`** (new): one small binary = message bus + memory store.
  - WebSocket transport (axum); rooms + DMs + presence; the 3 delivery modes reuse `parler-protocol`
    `Route` (Multicast = 1:many, Unicast = 1:1, Anycast/inbox = many:1).
  - **Memory** = embedded SQLite (rusqlite, bundled, FTS5): append-only message log per room +
    `facts` table w/ full-text recall + per-agent read cursors (agents fetch only new/relevant → low token cost).
  - **Pairing**: `invite` mints a token signed with the hub nkey (reuse `parler-auth`) → returns
    `parler://<hub>/join?c=…` or a short code; `join` redeems → durable member cred → auto-reconnect.
  - No external NATS / JWT operator chain in the MVP (those stay as a future pluggable transport).
- **`parler-connector`** (build out the stub): the `MeshAgent` client **core**, exposed through thin adapters.
  - `MeshTransport` trait: `HubClient` (WebSocket, MVP) now; `NatsTransport` (reuse existing work) later.
  - **CLI** (`parler` binary) **and** **MCP** (hand-rolled JSON-RPC-over-stdio — no heavy SDK) wrap the SAME core.
  - **Wake** = Claude Code `Stop` hook (pull inbox → continue the turn) + the Hermes `MeshHandle` seam
    already waiting in `parler-connect-hermes/serve.rs`. Hermes via its Python plugin.
  - **Durable connection**: persisted nkey creds (`~/.parler/`) + hub-side per-(agent,room) cursor ⇒ reconnect resumes.

### Phases
- [x] **P1 Hub core** — axum WS server; nkey challenge-response identity; rooms/membership/presence;
  the 3 delivery modes (room/dm/service) over WS; SQLite persistence + per-(agent,room) cursors.
- [x] **P2 Pairing** — invite mint/redeem (capability codes + links), durable membership, reconnect/resume.
- [x] **P3 Memory** — message log + FTS5 `facts`; `remember`/`recall` with scope (room vs private); cursors.
- [x] **P4 Client (CLI + MCP)** — `MeshAgent` core + `MeshTransport` + `HubClient`; the `parler` CLI
  (`hub`/`init`/`invite`/`join`/`serve`/`send`/`recv`/`remember`/`recall`/`rooms`/`roster`/`presence`/
  `whoami`) **and** `parler mcp` (hand-rolled stdio MCP server, 10 `parler_*` tools) over the SAME core.
- [~] **P5 Wake + polish** — quickstart docs done (`docs/agent-mesh.md`, incl. a drop-in Claude Code
  `Stop`-hook + MCP config). *Still open:* wiring the Hermes `MeshHandle` seam to the live client;
  optional live server push (`Subscribe`/`Delivery`); a demo traffic generator.

### Review — 2026-06-27
Built the focused "Slack for agents" feature end-to-end (no full Cotal/NATS copy).
- **New/changed crates:** `parler-protocol::hub` (shared frames); new `parler-hub` (server + SQLite/FTS
  store); built out `parler-connector` (MeshAgent/HubClient/Config), `parler-cli` (the `parler` binary +
  `mcp` module), `parler-bin`.
- **Model:** everything is a *room*; the 3 patterns are membership shapes. Pull + durable cursor (no live
  push yet) ⇒ stateless-per-message hub, trivially durable, reconnect-resumes.
- **Tests:** `cargo test` green for the feature crates — protocol 18, hub 6 (store/server unit incl. FTS
  recall + invite limits + cursor), connector 1 + **6 e2e** (`mesh_e2e.rs`: 1:1 / 1:many / many:1 /
  memory scope / reconnect-resume / non-member-denied). Real-process smoke test passed: 2 agents pair via
  a code, broadcast+receive, recall a fact, and the MCP server answers initialize/tools.list/tools.call.
- **Pre-existing failure (not this work):** `parler-auth/tests/auth_live.rs` needs a `nats-server` binary
  that isn't vendored here (`.context/bin/nats-server`); unrelated to the mesh feature.

> The waves below are the **original full-parity rewrite plan**, now **deprioritized** per the redirection.

---

# Parler — build tracker

Full-parity Rust rewrite of [Cotal](https://github.com/Cotal-AI/Cotal). Plan:
`~/.claude/plans/system-instruction-you-are-working-tender-wolf.md`. Reference clone:
`.context/cotal-ref/`. Local `nats-server`: `.context/bin/nats-server`.

## Wave 0 — Foundation
- [x] Cargo workspace + 15 crate skeletons (`crates/parler-*`), shared workspace deps, `.gitignore`
- [x] `parler-protocol`: wire types (`types.rs`) + subject grammar (`subjects.rs`), rebranded `cotal`→`parler`
- [x] Protocol tests: SPEC §12 subject vectors, matchers, collapse, mentions, member-key, envelope round-trip (15 passing)
- [ ] `parler-protocol`: `schemars` schema gen → `spec/parler.schema.json` + validation test
- [x] `parler-auth`: nkeys identity (`identity.rs`) — id/seed/creds parse
- [x] `parler-auth`: NATS decentralized JWT v2 issuance (operator→account→user) + creds format
- [x] `parler-auth`: six profile ACLs + `nats-server` config render
- [x] **De-risk:** boot real `nats-server` with minted JWTs; connect with minted user creds; assert allow/deny ✅ (tests/auth_live.rs)

## Wave 1 — Core (`parler-core`)
- [ ] connection (creds/open) + stream & KV provisioning (exact policies from `streams.ts`)
- [ ] presence (KV heartbeat + stale→offline sweep + roster + watch)
- [ ] three delivery modes (multicast/unicast/anycast) with subject-derived authenticated kind
- [ ] explicit ack-on-surface; dedup by id across paths
- [ ] channels registry + history backfill (`historical=true`, watermark ack-drop)
- [ ] Plane-3 durable membership + fan-out/reader/dlv + ACL re-auth
- [ ] per-module integration tests vs live broker

## Wave 2 — Surfaces & connectors (parallel)
- [ ] `parler-connector`: MeshAgent + 17 `parler_*` tools + orientation/relay/control/launch
- [ ] `parler-manager`: control-plane handler + PTY runtime + roster + spawn/despawn + MAX_AGENTS
- [ ] `parler-delivery`: daemon (fan-out + trusted reader + single-flight lease)
- [ ] `parler-cli`: all subcommands + YAML manifest engine + MeshView model
- [ ] `parler-console`: ratatui TUI (+ plain stream)
- [ ] `parler-web`: axum HTTP+SSE dashboard (+ static assets)
- [ ] `parler-connect-claude` (rmcp MCP + hooks + transcript)
- [ ] `parler-connect-opencode` (Rust sidecar + JS plugin shim)
- [x] `parler-connect-hermes`: bridge protocol + serial ack-on-surface state machine + launch recipe + Python plugin (11 tests); live mesh via the `MeshHandle` seam, pending `parler-connector`
- [x] `parler-core` Runtime/Terminal/Launch contracts (the host-integration traits cmux/tmux/manager share)
- [x] `parler-cmux` driver (8 tests: CLI wrapper, temp-script gen, layout, id/ref parsing)
- [ ] `parler-tmux` driver (mirror of cmux over the tmux CLI)
- [ ] `parler-bin`: compose all subcommands into the `parler` binary

## Wave 3 — Integration & polish
- [ ] Full conformance suite (14 §12 MUSTs + interop scenario)
- [ ] Port the ~50 `*.smoke.ts` integration tests
- [ ] `demo` traffic generator
- [ ] Benchmarks vs Node (`criterion` + e2e RTT/throughput/memory) → `docs/benchmarks.md`
- [ ] docs / examples / Docker / release packaging

## Review
- 2026-06-24: Foundation + auth landed. `cargo test --workspace` green = **24 tests**
  (15 `parler-protocol` + 8 `parler-auth` unit + 1 live broker integration).
  - `parler-protocol`: untagged `Route` + `#[serde(flatten)]` emits exactly one of
    `channel`/`to`/`toService`; SPEC §12 subject-parse vectors pass.
  - `parler-auth`: hand-rolled NATS JWT v2 (operator/account/user) since `nats-jwt` lacks operator +
    JetStream limits. **Top risk retired**: `tests/auth_live.rs` boots the real `nats-server`, mints
    creds, and the broker enforces the agent ACL (declared-channel publish delivered; undeclared
    rejected) and account JetStream (manager creates the CHAT stream).
  - **Next:** `parler-core` endpoint (port the 133 KB `endpoint.ts`) — connection + stream/KV
    provisioning + presence + the three delivery modes, then the §12 interop scenario as the
    foundation-slice e2e (task #5).
- 2026-06-24: cmux + hermes parity. `cargo test --workspace` = **43 tests** green (added 8 cmux + 11
  hermes + the parler-core contracts). Added the `parler-core` host-integration contracts
  (Runtime/AgentHandle/Terminal/Launch) — Rust uses explicit construction, not the TS global Registry.
  - `parler-cmux`: full cmux CLI driver + Runtime + TerminalLayout; pane temp-script + layout JSON
    + workspace id/ref parsing all tested without a live cmux.
  - `parler-connect-hermes`: the bridge **wire protocol** + the serial **ack-on-surface** state
    machine (incl. the in-flight-eviction edge case) + the **launch** recipe, all tested; the
    **Python plugin** ported faithfully under `plugin/parler/` (adapter/hooks/tools/bridge_client,
    rebranded). The live mesh plugs into the `MeshHandle` trait in `serve.rs` once `parler-connector`
    lands; the unix-socket server is compiled glue around the tested state machine.
