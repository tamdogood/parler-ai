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
