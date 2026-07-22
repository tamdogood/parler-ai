# Beads, Gas Town, and Parler Protocol: technical study and improvement roadmap

- Date: 2026-07-21 (America/Denver; some upstream commits are dated 2026-07-22 UTC)
- Parler baseline: `49a51dc`, the then-current `origin/main`
- Audience: Parler maintainers and future product/engineering contributors
- Scope: research and recommendations only; no product, API, protocol, schema, or backlog changes

## Executive conclusion

Parler should not become Beads, Gas Town, or Gas City. It should become the authenticated,
independent-agent communication plane that none of those systems makes its core authority.

The ecosystem separates into four useful categories:

| System | Category | Authority it should retain |
|---|---|---|
| **Beads** | Durable agent-oriented work and memory ledger | Issues, dependencies, readiness, ownership, workflow state, and Dolt history |
| **Gas Town / Gas City** | Agent orchestration control plane | Dispatch, runtime/session lifecycle, supervision, formulas, recovery, and merge policy |
| **Wasteland** | Asynchronous federated work market | Cross-organization work records, evidence, reconciliation, and eventual reputation |
| **Parler Protocol** | Authenticated live communication plane | Self-certifying identity, signed target-bound messages, rooms, durable receive cursors, visible-host turns, and signed handoffs |

This distinction is not semantic hair-splitting. Each system makes a different thing authoritative.
Beads makes a local-first database authoritative. Gas City makes a controller and its work queries
authoritative. Wasteland makes independently replicated work records and later reconciliation
authoritative. Parler makes a hub-backed conversation log, self-certifying senders, and receiver
cursors authoritative.

The recommended strategy is:

1. **Borrow the design patterns, not the category.** Adopt Beads' explicit state/authority
   boundaries, local-versus-durable split, atomic claim discipline, migration hygiene, and
   context-on-demand lessons. Do not import its issue ontology or Dolt into Parler core.
2. **Integrate at references and transport seams.** First prove an external adapter that correlates
   an opaque Beads task reference with Parler's existing signed handoff and receipt messages. If
   users pull that bridge into real workflows, implement a Gas City external-messaging transport
   adapter that maps a Parler conversation to a Gas City conversation. Gas City should keep
   control of its sessions and wake path; a Pack may package and configure the adapter.
3. **Differentiate on trust and live delivery.** Parler's defensible assets are self-certifying
   identity, signed target-bound messages, durable acknowledgement cursors, portable live
   conversations, content-addressed artifacts, and visible turns in supported hosts.
4. **Avoid storage and orchestration expansion.** A task graph, formula engine, Dolt replication,
   agent supervisor, merge queue, or work marketplace would duplicate mature external systems and
   weaken Parler's small beginner model.

The best near-term interoperability choice is therefore an **additive Beads task-reference
adapter**, prototyped outside the protocol. It can persist a mapping from the originating Parler
message ID to the external task reference, then resolve built-in receipts through that mapping.
`Part::Extension` provides an optional additive namespace if readable text plus correlation proves
insufficient. No wire change is justified before a prototype demonstrates demand.

## Evidence and interpretation

### Status labels

This report uses five status labels:

- **Stable-release**: observed in the named release's code, release notes, or downloaded binary.
- **Current-main**: observed at the pinned `main` commit; it may not be in a stable release.
- **Historical**: a prior behavior, incident, or maintainer position that explains current design.
- **Planned**: documented intention without a verified shipped implementation.
- **Inferred**: a conclusion drawn from observed facts, explicitly identified as such.

“Observed” means code, tests, a direct command result, or first-party documentation supports the
claim. It does not mean the authors' marketing interpretation was accepted uncritically. Important
negative claims were searched across the relevant repositories; where no second independent origin
exists, the report says so rather than treating absence as proof.

### Version snapshot

The exact upstream references inspected are:

| Project | Latest stable/tag at research time | Stable commit | Pinned current `main` | Status note |
|---|---|---|---|---|
| [Beads](https://github.com/gastownhall/beads) | [`v1.1.0`, 2026-07-04](https://github.com/gastownhall/beads/releases/tag/v1.1.0) | [`8e4e59d3`](https://github.com/gastownhall/beads/tree/8e4e59d39f3459a43cf21a3236a13eca4dd874f7) | [`607df586`, 2026-07-22 UTC](https://github.com/gastownhall/beads/tree/607df586d198400e2a65681b824b7d6e21a14d97) | Main was 441 commits past stable |
| [Gas Town](https://github.com/gastownhall/gastown) | [`v1.2.1`, 2026-06-06](https://github.com/gastownhall/gastown/releases/tag/v1.2.1) | [`319d33a9`](https://github.com/gastownhall/gastown/tree/319d33a91b2deca59bba6dd26be6b9daf8eaacf6) | [`67a8d72a`, 2026-07-20](https://github.com/gastownhall/gastown/tree/67a8d72a7aa415cad5b9832bdbba31b6ec026417) | Active orchestrator; not formally deprecated |
| [Gas City](https://github.com/gastownhall/gascity) | [`v1.3.5`, 2026-07-14](https://github.com/gastownhall/gascity/releases/tag/v1.3.5) | [`8ffc009d`](https://github.com/gastownhall/gascity/tree/8ffc009ded781a2ada2077f3a29bd712b2def0bf) | [`2abd12e8`, 2026-07-22 UTC](https://github.com/gastownhall/gascity/tree/2abd12e857a2c38875db51b681736a4e053b89b1) | Reusable machinery extracted from Gas Town |
| [Gas City Packs](https://github.com/gastownhall/gascity-packs) | No GitHub release; latest tag `v0.3.0`, 2026-06-11 | [`9d53ae5f`](https://github.com/gastownhall/gascity-packs/tree/9d53ae5f43efa1cb4c0c69a4bf5fd6730cfbb5b7) | [`56d07c53`, 2026-07-15](https://github.com/gastownhall/gascity-packs/tree/56d07c537b21d9db239ed6f9e17e6c6e37da0864) | Tag, not a declared stable release |
| [Wasteland](https://github.com/gastownhall/wasteland) | [`v0.4.0`, 2026-03-08](https://github.com/gastownhall/wasteland/releases/tag/v0.4.0) | [`5535bcdc`](https://github.com/gastownhall/wasteland/tree/5535bcdc3a368df0d4c9025dcdccc237ccf49e6e) | [`fbad824c`, 2026-07-07](https://github.com/gastownhall/wasteland/tree/fbad824c1e826be00ec7e8434473e6a70556a6fd) | Phase and backend qualifications matter |

Annotated tags were peeled to their commits. The Gas City Packs `v0.3.0` and Wasteland `v0.4.0`
tags are lightweight; the table records their direct target commits.

### Research method

The study used five evidence layers:

1. Source and tests at the exact commits above.
2. Stable release notes, checksums, and locally executed release binaries.
3. Parler implementation and tests at `49a51dc`, rather than roadmap claims alone.
4. Package-manager and repository snapshots, kept separate because their populations overlap.
5. An adversarial pass over maintainer positions, incidents, disputed claims, and reasons not to
   copy a design.

Raw notes and command output remain in the locally ignored `.lead/research/` directory. Only this
synthesis is committed.

### Major finding status map

| Finding | Status | Evidence basis |
|---|---|---|
| Four-category system map | **Inferred** | Pinned charters/code paths plus local workflows |
| Beads issue/dependency/ready/claim/close model | **Stable-release and current-main** | `v1.1.0` code/binary lab; pinned main source |
| Beads node-local lease/heartbeat/reclaim and row-version work | **Current-main** | Pinned main source/tests; not claimed for stable |
| Beads Dolt authority and migration procedure | **Stable-release** with **historical** incident drivers | `v1.1.0` release/code plus two-clone lab |
| Beads formulas/molecules/wisps/gates | **Stable-release and current-main** | Stable CLI/code and pinned main model |
| Gas Town as an opinionated orchestrator | **Stable-release and current-main** | `v1.2.1` binary; pinned architecture |
| Gas City as extracted platform/control plane | **Stable-release and current-main** | `v1.3.5` CLI dry-run; pinned dispatch/session code |
| Gas City external messaging fabric | **Stable-release and current-main** | Stable source/OpenAPI; pinned main source/tests |
| Gas City connected external LLM client over SSE | **Planned/spec-first** | Pinned guide plus release gate stating endpoint implementation is absent |
| Gas City signed per-city API admission grants | **Current-main** | Pinned read/write middleware, config, and tests; not claimed for stable |
| Wasteland wild-west limitations | **Historical/current documentation for that mode** | Pinned Gas Town Phase 1 doc |
| Wasteland PR-mode branch workflow | **Stable-release and current-main** | `v0.4.0` local workflow; pinned SDK |
| Wasteland optional GPG-signed Dolt history | **Stable-release and current-main implementation** | Stable source/help and pinned verify command; not exercised in the lab |
| Parler capability baseline | **Shipped at Parler `49a51dc`** | Local code/tests, reconciled against docs/backlog |
| Adoption numbers | **Dated current snapshot** | GitHub and Homebrew, never summed |
| Integration interfaces and roadmap | **Proposed/planned** | Inference from shipped seams; none is claimed implemented |

## The systems, end to end

```text
                                      asynchronous work federation
                                +--------------------------------------+
                                | Wasteland + DoltHub forks/reconcile  |
                                +------------------+-------------------+
                                                   ^
                                                   | work records/evidence
                                                   |
+----------------------+       queries/updates     v       starts/observes
| Beads work ledger    | <----------------> +---------------------------+
| issues, deps, ready  |                     | Gas Town / Gas City      |
| claims, memory, Dolt |                     | dispatch, runtime, health |
+----------+-----------+                     | formulas, merge policy    |
           ^                                 +-------------+-------------+
           | durable state                                 |
           |                                               | optional adapter
           | task references/status                        v
           |                                  +---------------------------+
           +--------------------------------- | Parler communication plane |
                                              | signed rooms, cursors,     |
independent agent hosts <---- visible turns --| handoffs, files, memory    |
                                              +-------------+-------------+
                                                            |
                                                            v
                                               one authoritative SQLite hub
```

The diagram shows the recommended boundary, not a shipped integration. Today these projects do not
form one product.

### What “messaging” means in each system

- **Beads stable/current-main:** a message can be an issue record with sender/thread fields, and
  `bd mail` delegates to an external command. The database preserves message-shaped state, but
  Beads does not itself establish network delivery, presence, wake, or cryptographic authorship.
  See the [current messaging design](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/engdocs/messaging.md)
  and [mail delegation](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/cmd/bd/mail.go).
- **Gas City stable/current-main:** with the default `beadmail` provider, internal mail is a durable
  Beads issue; the alternate `exec` provider delegates persistence and delivery semantics to an
  operator script. Nudge is a best-effort runtime-provider call, and events are a separate
  observation stream. Gas City also ships an external messaging fabric with provider-neutral
  conversations, Beads-backed transcripts/membership cursors, adapter ingress/egress, and session
  routing. A bearer-authenticated external LLM client with SSE reply replay is documented but
  explicitly held as **spec-first/planned**, not shipped.
  See
  [messaging](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/messaging.md),
  the [stable fabric design](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/engdocs/design/external-messaging-fabric.md),
  and [current connected clients](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/docs/guides/connected-clients.md).
- **Parler shipped:** a message is an author-signed, target-bound record in a room, DM, or service
  log. Push and long-poll reduce latency, while the persisted log and deferred member cursor remain
  durable receiver-state truth within each consumer's documented boundary. Supported visible
  adapters turn verified peer messages into normal host turns.
  See [the wire types](../../crates/parler-protocol/src/hub.rs),
  [connector receive/ack](../../crates/parler-connector/src/agent.rs), and
  [visible host contract](../visible-host-adapters.md).

### What “memory” means

- **Beads:** structured operational memory. Durable issues, decisions, dependencies, comments,
  events, compacted history, and keyed `bd remember` facts reappear through `bd prime`.
- **Gas Town / Gas City:** persisted work and default-provider mail survive runtime death, so a
  restarted worker can re-read its hook/query and resume. The `exec` mail provider can choose
  different semantics. The control plane's memory is mostly work-state recovery.
- **Parler:** scoped facts are separate from conversation history and support exact keyed lookup,
  FTS5/BM25, and sqlite-vec hybrid retrieval. Parler also retains room messages and
  content-addressed artifacts, but does not pretend those are a task graph.

### What “federation” means

- **Beads:** Dolt remotes copy and merge database history. Stable `v1.1.0` and current-main expose
  peer configuration and `bd federation sync` over those stores. This federates work state, with
  database conflicts and migrations as operational facts.
- **Wasteland:** sovereign forks exchange work-market state through DoltHub and reconcile later.
  Current Phase 1 claims are not distributed locks.
- **Parler:** no cross-hub federation is shipped. A `KEY@HUB` descriptor routes a joiner to one
  authoritative hub; it does not replicate history or establish global discovery.

### What “multi-agent” means

- **Beads:** many actors can inspect and mutate a shared graph; claims coordinate ownership.
- **Gas Town / Gas City:** a controller launches, routes, watches, restarts, and composes multiple
  coding-agent runtimes.
- **Parler:** independent agents with separate identities and hosts share authenticated
  conversations and handoffs. Parler's role queues can select one worker, but Parler is not a
  general scheduler or workflow graph.

## Beads: codebase and design

### Category and charter

Beads' current [project charter](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/engdocs/PROJECT_CHARTER.md)
is the most useful scope document. It says core owns issue lifecycle, dependencies, readiness,
metadata, comments, local CLI, import/export, sync, backup/recovery, and tracker mapping. It
explicitly excludes agent routing, task-assignment strategy, model choice, retries, scheduling,
workflow semantics, and cross-system coordination. It also says new concepts should usually begin
as metadata rather than schema.

That charter narrows earlier broad marketing. The accurate category statement is:

> Beads is a durable, distributed graph issue tracker and structured memory system optimized for
> agents. It can store messages and workflow-shaped records, but it is not an authenticated
> realtime transport or an agent runtime.

This is both **current-main observed** and consistent with the stable `v1.1.0` CLI. Calling it “only
a task tracker” would undersell it; calling it an orchestration or messaging protocol would
overstate it.

### Implementation map and architectural seams

The stable Go codebase is layered, but several generations of storage design coexist:

- [`cmd/bd`](https://github.com/gastownhall/beads/tree/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/cmd/bd)
  owns Cobra commands, argument policy, store selection, routing, rendering, and command-level
  follow-up writes/commits.
- [`internal/types`](https://github.com/gastownhall/beads/tree/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/types)
  defines the broad issue/dependency/formula wire and domain vocabulary, while the large
  [`Storage` interface](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/storage.go)
  is the main command-facing persistence seam.
- Shared [`issueops`](https://github.com/gastownhall/beads/tree/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/issueops)
  centralizes transactional mutations, and
  [`sqlbuild`](https://github.com/gastownhall/beads/tree/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/sqlbuild)
  centralizes important query construction such as readiness.
- [`embeddeddolt`](https://github.com/gastownhall/beads/tree/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/embeddeddolt)
  and [`dolt`](https://github.com/gastownhall/beads/tree/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/dolt)
  implement embedded and SQL-server-backed access. Version-control operations, remote/bootstrap
  logic, and schema repair sit beside them rather than behind a remote service.
- Formula/molecule packages, tracker adapters, hooks, and the Python MCP wrapper are edge layers
  over that same issue/store authority. They do not create a second canonical task model.
- Domain/use-case and unit-of-work packages plus the per-workspace proxied Dolt server show a move
  toward more explicit transactional/server boundaries. The stable CLI labels proxied-server mode
  **experimental** in its
  [init flags](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/cmd/bd/init.go),
  so it should not be treated as the default architecture.

Schema surface is correspondingly large. The stable
[migration tree](https://github.com/gastownhall/beads/tree/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/schema/migrations)
contains tracked migrations through `0053` and Dolt-ignored local-state migrations through `0011`;
the pinned [current tree](https://github.com/gastownhall/beads/tree/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/schema/migrations)
reaches `0055` and ignored `0013` for leases/row locks. Backend parity therefore requires more than
interface conformance. The stable suite includes
[backend conformance](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/dolt/conformance_test.go),
[pull-conflict](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/dolt/pull_conflict_test.go),
remote-migration, schema-skew, concurrency, and cross-upgrade tests, plus the migration-hygiene
script. This breadth is a strength and a cost: every new store path multiplies parity, migration,
recovery, and help-text obligations.

**Lesson for Parler:** keep `MeshTransport` as a narrow behavioral seam with one authoritative
message/cursor store. Add conformance tests for adapters, but do not multiply canonical storage
backends merely to gain offline merge semantics.

### Core model: one broad issue graph

The current [`types.Issue` model](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/types/types.go)
combines:

- issue content, status, priority, type, assignee, owner, and timestamps;
- due/defer scheduling and external tracker references;
- arbitrary JSON metadata;
- labels, dependencies, comments, events, compaction, and snapshots;
- messaging, sender, ephemeral/no-history, gate, molecule, swarm, and wisp fields;
- current-main optimistic concurrency through a row-version token.

IDs use a project prefix plus an adaptive base-36 hash suffix. Child work uses hierarchical IDs.
They are decentralized and collision-resistant, but they are not pure content addresses.

Dependency direction is explicit: **the issue depends on the blocker**. `bd dep add implement
design` means implementation cannot become ready until design closes. Blocking relations include
ordinary blocks, parent-child propagation, conditional blocks, and waits-for. Other relations such
as related, replies-to, discovered-from, duplicates, validates, or delegated-from enrich the graph
without necessarily blocking readiness.

Readiness is a materialized operational query, not a planner. The default excludes closed, blocked,
future-deferred, pinned, ephemeral, and infrastructure/workflow roots, then applies label,
assignee, parent, molecule, priority, type, and metadata filters. Current-main centralizes much of
that query in
[`sqlbuild/ready.go`](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/sqlbuild/ready.go).

**Lesson for Parler:** precise dependency direction, ready criteria, and authority are good design.
The graph itself is not. Parler handoffs should reference an external unit of work and report
communication lifecycle; they should not acquire hidden blocker semantics.

### Claims, leases, and concurrency

Stable `v1.1.0` provides atomic conditional claims. In the hands-on race, two simultaneous
`bd ready --claim` calls targeted the same queue; one actor received the item and the other received
an empty result. Re-claiming the same known issue by the same owner is idempotent. A fresh
`ready --claim` is queue acquisition without a stable request/item identity; a retry may return
nothing or claim another ready item.

Current-main goes further:

- durable claimed work receives a short lease;
- a heartbeat renews a node-local `leases` row without minting Dolt history;
- reclaim can return expired ownership to ready state;
- lifecycle/ownership writes update a shared `row_lock` cell so Dolt's cell-level merge does not
  silently combine logically conflicting writes;
- guarded update/close exposes row-version compare-and-swap behavior.

These features are visible in current-main
[lease operations](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/issueops/lease.go),
[Dolt issue operations](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/dolt/issues.go),
and [lease tests](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/dolt/lease_test.go).
They are **current-main**, not part of the stable binary experiment.

The local/durable split matters. Status and assignee replicate, but heartbeat state does not. A
remote clone sees ownership without sharing the granting node's live lease. Dolt's SQL parser
accepts familiar lock syntax, but Beads' comments state that real row locking is absent; correctness
comes from transactions, serialization retries, compare-and-set checks, and the shared conflict
cell.

**Lesson for Parler:** keep high-frequency liveness local while durable state remains auditable.
Parler already follows this principle when push acts only as a doorbell and the durable cursor
remains truth. The existing service claim lease and any future lease or liveness changes must keep
the same explicit authority boundary.

### Dolt as canonical storage

Beads has moved beyond its historical SQLite/JSONL architecture. At stable `v1.1.0`:

- Dolt is canonical.
- Embedded Dolt is the ordinary local mode.
- `dolt sql-server` is available for multi-process/multi-writer use.
- JSONL is interchange, export, and viewer material, not cross-machine source of truth.
- push and pull are explicit; remotes may use file, Git, DoltHub, S3, or GCS-oriented paths.
- ordinary work can continue offline.

Dolt gives Beads valuable properties: queryable structured state, commits, time travel, branching,
cell-level merge, and remote replication. It also makes database schema part of distributed shared
history.

That second property drove significant operational machinery. The
[`v1.1.0` release notes](https://github.com/gastownhall/beads/releases/tag/v1.1.0) center on
migration hashes, remote migration gates, schema drift repair, dependency-key repair, foreign-key
repair, and a **single designated migrator** procedure. Other clones must adopt or bootstrap the
migrated history instead of independently applying ambiguous migrations. The stable CI check
rejects duplicate migration numbers and nondeterministic migration SQL through the
[migration-hygiene check](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/scripts/check-migration-hygiene.sh).

This is not an argument that Dolt failed. The hands-on merge behaved as advertised: edits to
different fields merged; edits to the same field stopped for operator resolution and restored the
working set. It is an argument that these benefits come with version-skew, bootstrap, credential,
schema-history, and conflict-recovery costs.

**Decision for Parler:** do not replace the hub's SQLite message/cursor authority with Dolt. Message
delivery needs one authoritative append order and clear acknowledgement truth, not peer database
merge.
Dolt remains appropriate behind a work ledger that users may choose to integrate.

### Formulas, molecules, wisps, gates, and swarms

These terms are easy to mistake for an agent runtime:

- A **formula** is a reusable workflow template with variables, validation, steps, needs,
  conditions, loops, gates, composition points, and phases.
- A **molecule** is an instantiated root-plus-child DAG represented in the issue/dependency graph.
- A **wisp** is transient workflow-shaped state in Dolt-ignored local tables. Burn discards it
  without a durable digest; squash creates a permanent digest before deleting it; promotion moves
  selected state into ordinary durable issues and versioned history.
- A **gate** records an await condition, external identifier, timeout, and waiters. Some conditions
  can be checked; others require external or human action.
- A **swarm** validates an epic graph, derives parallel `ReadyFront` work waves, and can record a
  separate linked swarm molecule. It does not spawn or supervise agents.

The stable code routes ephemeral/no-history records into wisp tables and skips ordinary Dolt
commits. Current-main continues this deliberate split. Gas Town/City consumes these graph
structures as a control plane; Beads itself does not choose models, schedule turns, or retry
runtimes.

**Lesson for Parler:** typed, inspectable handoffs are valuable. A workflow language in the wire
protocol is not. Parler should carry a reference, instruction, status, and artifact while an
external ledger/control plane owns graph semantics.

### Memory, prime, hooks, and MCP

`bd remember` stores bounded project memory that `bd prime` injects into an agent's context.
`bd prime` also summarizes the current workflow and state, with host hooks for session start and
post-compaction reinjection. This is a practical answer to context loss: keep source-of-truth state
outside the model, then present the smallest timely slice.

Beads' MCP integration is a Python FastMCP server that discovers tools lazily and shells out to the
`bd` CLI rather than reimplementing the database. The stable implementation is visible in
[`server.py`](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/integrations/beads-mcp/src/beads_mcp/server.py)
and [`bd_client.py`](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/integrations/beads-mcp/src/beads_mcp/bd_client.py).
The boundary keeps one CLI behavior surface, though it inherits CLI startup, discovery, version,
and mode differences.

Database lifecycle hooks fire after commit in the ordinary direct-store path and are generally
asynchronous/best-effort. Git and host hooks use managed sections so existing hooks can be chained.
`bd init` can update `AGENTS.md` and install host/Git integration unless users select
`--skip-agents`, `--skip-hooks`, or `--stealth`.

**Lessons for Parler:**

- Re-inject compact, current state after context compaction instead of expanding every tool result.
- Prefer one implementation path behind CLI/MCP adapters.
- Make setup side effects explicit and reversible.
- Measure Parler's current MCP concept/tool load before adding any integration tools.

### Security boundary

Beads' [security guidance](https://github.com/gastownhall/beads/security) says issue data is
plaintext and the product does not provide database encryption or an access-control system.
Assignee and actor values are strings, not cryptographic principals. Peer credentials receive
local protection: pinned current-main's
[`credentials.go`](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/dolt/credentials.go)
uses AES-GCM with a random local key written mode `0600`. This reduces at-rest credential exposure;
it does not encrypt issue content, authenticate database authors, or bind an issue mutation to an
agent key.

Stable `v1.1.0` added opt-out Beads metrics; the stable
[metrics code](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/metrics/metrics.go)
recognizes `BD_DISABLE_METRICS`. The current security page simultaneously says the Beads codebase
has no telemetry while warning about Dolt telemetry. That is documentation drift, not a reason to
infer malicious collection.

Beads has also hardened initialization, secret handling, remote URL validation, and destructive
operations after concrete incidents. The stable
[changelog](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/CHANGELOG.md)
records an AI-caused loss of 247 issues as one init-safety driver. The right lesson is to preserve
explicit destructive boundaries and recovery paths, not to claim that work-ledger automation is
inherently unsafe.

### Governance and operating history

Current [contribution rules](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/CONTRIBUTING.md)
and [maintainer guidelines](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/PR_MAINTAINER_GUIDELINES.md)
are unusually explicit about one issue per PR, tests/docs/CI, contributor attribution, and never
silently replacing a contributor's work. The guidelines report an internal audit of 440 merged PRs
whose worst defects came from skipped, hidden, or overridden review. That is a maintainer-origin
claim, but the resulting policy is directly observable.

For Parler, the transferable lesson is procedural: agent-generated volume does not reduce the need
for small diffs, visible provenance, negative tests, and human-readable review. Parler's existing
engineering and review guidelines already align with this.

## Hands-on stable Beads validation

### Safety and provenance

All experiments ran in a disposable temporary directory. Nothing was installed globally, no real
coding agent was launched, no user host configuration or hooks were changed, and no GitHub or
DoltHub state was mutated.

The downloaded macOS arm64 `v1.1.0` archive was checked against the release checksum:

```text
asset:    beads_1.1.0_darwin_arm64.tar.gz
sha256:   c42e24d83b258f7ba9f52a6d2d5f6b055869dfe7807165055988b12e7ea8c564
version:  bd version 1.1.0 (8e4e59d39)
```

Beads telemetry was disabled with `BD_DISABLE_METRICS=1`. Repositories were initialized with
`--non-interactive --stealth --skip-agents --skip-hooks`. This validates the release's opt-out and
the non-invasive initialization path; it does not validate that every downstream dependency's
telemetry was disabled.

### Workflow results

| Experiment | Stable-release observation | Architectural conclusion |
|---|---|---|
| Create `design`, `implement`, `verify` with a dependency chain | Only `design` appeared in `bd ready` | Readiness follows blocker direction and graph state |
| Race two `bd ready --claim --json` actors | Actor A claimed the one ready issue; actor B received no issue | Stable conditional claim prevents two winners on one store |
| Close `design` | Close output named `implement` as newly unblocked; `ready` returned it | Close and downstream readiness update are connected |
| `bd remember`, recall, and `bd prime` | Stored memory was recalled and included in prime output | Memory is durable project context, not chat recall |
| Create an ephemeral heartbeat wisp | Wisp was usable; `bd history` reported no history | Transient workflow state intentionally avoids durable history |
| Push to a local `file://` remote and initialize a second clone | The second clone bootstrapped and adopted the same project | Dolt history, not JSONL, is the replication authority |
| Edit the same notes cell in both clones | Pull exited with an issue conflict, aborted the merge, restored the working set, and retained the clone's edit | Same-cell conflict requires an operator; data was not silently discarded |
| Edit acceptance in one clone and design in another | Push/pull succeeded and both fields remained | Dolt can merge non-overlapping cell edits |
| Run embedded `bd sql` | Stable binary rejected the operation as unsupported although generic help advertised SQL | Mode/help parity is imperfect; this is a product seam, not a category failure |

Current-main leases were source- and test-validated, but not backported into this stable experiment.
The report does not present them as a `v1.1.0` behavior.

### Stable create code path

A canonical `bd create` operation was traced at the stable release:

1. [`cmd/bd/create.go`](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/cmd/bd/create.go)
   validates arguments, builds `types.Issue`, parses parents/dependencies, and calls the store.
2. Default embedded mode enters
   [`embeddeddolt/create_issue.go`](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/embeddeddolt/create_issue.go).
3. Shared
   [`issueops/create.go`](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/issueops/create.go)
   validates/defaults the record, generates an ID, inserts the issue, and records associated
   events, labels, and comments in one SQL transaction.
4. The CLI then adds parent, dependency, and waits-for edges through separate `AddDependency`
   calls. A failed edge logs a warning and does not roll back the already-created issue.
5. After the dependency loop, default embedded auto-commit mode calls `store.Commit`; the common
   post-run path may then find no pending changes. Wisp/no-history paths intentionally skip an
   ordinary Dolt history commit.

This trace supports a narrower architectural conclusion: issue creation is a local SQL transaction
followed by best-effort graph-edge writes and a later Dolt history commit. `bd create --deps` is not
one atomic graph mutation, and a crash or warning boundary exists after the issue row. Network
delivery is not on any of those paths.

## Gas Town and Gas City: the orchestration control plane

### Relationship and scope

Gas Town is an opinionated multi-agent workspace/orchestration system. Its
[architecture](https://github.com/gastownhall/gastown/blob/67a8d72a7aa415cad5b9832bdbba31b6ec026417/docs/design/architecture.md)
defines named roles, rigs/worktrees, work queues, mail, session processes, health patrol, and a
Refinery merge path.

Gas City extracts the reusable machinery into a configurable platform: agents, runtime providers,
work queries, formulas, orders, services, hooks, health/reconciliation, and packs. The
[migration guide](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/docs/getting-started/coming-from-gastown.md)
says Gas Town is included among orchestrators built on that platform. No primary source formally
deprecates Gas Town. The accurate statement is therefore:

> Gas City is the extracted platform and architectural successor path; Gas Town remains a concrete,
> opinionated orchestrator above related machinery.

Parler should integrate with the platform seam or an external Pack, not copy the named-town
ontology into its protocol.

### One dispatch, traced

The current-main dispatch path was traced from CLI to runtime boundary:

1. `gc sling` enters
   [`cmd/gc/cmd_sling.go`](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/cmd/gc/cmd_sling.go)
   through `cmdSling`, then `doSlingBatch`, then `doSling`.
2. Gas City resolves the city, target agent, Beads store, and optional container/rig context.
3. A formula dispatch may cook a molecule/wisp and select its root as the work record.
4. Preflight checks inspect the bead and target. The agent's `EffectiveSlingQuery` substitutes the
   bead ID into its configured routing command.
5. An injectable `SlingRunner` executes the routing operation. Current tests replace this with a
   fake runner, which is the important testability seam.
6. The path records telemetry and merge/convoy metadata.
7. Only after the durable route succeeds does an optional `doSlingNudge` invoke the runtime
   provider.

The [dispatch design](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/dispatch.md)
and implementation agree on the authority sequence: durable work first, wake second. Pool demand
and worker work queries share the same routing label, so the controller starts capacity for the
same queue workers later claim.

This is not exactly-once agent execution. A routed record can outlive a process, a nudge can be
lost, and recovery can start a fresh process that re-reads persistent work. The design is
retry-capable and recoverable, with application-level guards; the inspected sources do not prove
that an execution attempt eventually occurs.

### Mail, nudge, and events are different mechanisms

| Mechanism | Persistence | Delivery/wake guarantee | Role |
|---|---|---|---|
| **Mail (`beadmail`)** | Beads issue of type `message` | Durable shared state; read is a label and archive/delete closes the record | Default inbox, threads, instructions that must survive |
| **Mail (`exec`)** | Provider-defined | An operator-supplied script owns persistence/delivery semantics | Custom mail integration |
| **Nudge** | None | Fire-and-forget; explicitly lost if the session is absent | Low-latency hint to a running runtime |
| **Event** | Append-only sequence stream | Recording is best-effort; errors are logged and generally not returned | Observation, UI, controller and audit signals |

This split is documented directly in Gas City's
[messaging architecture](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/messaging.md)
and [event-bus architecture](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/event-bus.md).

The split explains both the product's resilience and its concept load. Default durable mail is not a
realtime bus. Nudge is not recoverable delivery. Events are not commands. A user must understand
which one they are using.

**Lesson for Parler:** signed messages persist in one room log; push/long-poll are only doorbells;
deferred acknowledgement preserves receiver truth. Do not add a second “nudge message” that
bypasses that log. However, the
mail/nudge/event split is not Gas City's whole messaging story: its external messaging fabric also
has a durable transcript and per-session read cursor. Parler must differentiate on agent-owned
signatures, room target binding, portable independent-host membership, and content-addressed
artifact verification, not claim the log-plus-doorbell pattern alone.

### Stable external messaging fabric

Gas City `v1.3.5` already ships `internal/extmsg`, a provider-neutral bridge between external
conversations and Gas City-controlled sessions. The
[stable design](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/engdocs/design/external-messaging-fabric.md)
and [shared-thread update](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/engdocs/design/external-messaging-shared-threads.md)
are marked implemented. The stable
[types](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/internal/extmsg/types.go),
[HTTP handlers/tests](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/internal/api/handler_extmsg.go),
and [OpenAPI document](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/docs/reference/schema/openapi.json)
confirm the shipped surface.

The fabric provides:

- a provider-neutral `ConversationRef` containing scope, provider, account, conversation, parent,
  and DM/room/thread kind;
- one durable active conversation-to-session binding, plus a scoped outbound delivery route;
- adapter registration and capability reporting;
- a raw in-process adapter verification/normalization seam, a controller-API pre-normalized ingress
  path, provider-message-ID deduplication, outbound publication, and returned publish receipts;
- Beads-backed transcript records with a monotonic per-conversation sequence;
- per-session conversation membership, backfill policy, `last_read_sequence`, list-backfill, and
  acknowledgement;
- group participants and speaker-selection policy;
- best-effort normalized-path session reminder/wake plus durable transcript backfill for
  successfully appended, routed entries.

Stable typed APIs expose adapter registration, bind/unbind, groups/participants, inbound/outbound,
transcript reads, and transcript acknowledgement under each city.

Registration itself is controller-lifetime state. Stable
[`adapter_registry.go`](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/internal/extmsg/adapter_registry.go)
says external adapters must re-register after controller restart, and unregister does not drain
already in-flight calls. Durable transcript/binding state does not remove that operational
reconciliation requirement.

One stable inbound path is:

1. a raw in-process transport adapter may verify a provider payload and normalize it to
   `ExternalInboundMessage`; alternatively, a caller posts an already-normalized message through
   the controller API. The stable HTTP adapter reports raw verification as unsupported, and the
   stable handler treats the normalized route as controller authority without authenticating the
   external provider. Stable admission relies on the listener/network boundary plus
   `X-GC-Request` anti-CSRF; current-main may additionally require a configured signed write grant;
2. [binding/group routing](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/internal/extmsg/inbound.go)
   first resolves a public responder and target session. Unroutable input returns without appending
   a transcript entry;
3. for routed input, the fabric deduplicates by `ProviderMessageID` and attempts to append a
   sequenced Beads-backed transcript entry. `DedupKey` exists in the type but was not consumed by
   the runtime at either pinned commit;
4. after normalized HTTP handling, the bridge sends a best-effort member reminder. A successfully
   appended entry remains available for backfill if the reminder fails. `ErrHydrationPending` is
   an explicit exception: no transcript entry is written, processing continues, and the normalized
   handler may still notify. The raw path does not run this member notifier, and the shipped HTTP
   adapter rejects raw verification;
5. members can backfill the transcript, and an explicit acknowledgement API advances a
   membership's `last_read_sequence`; the inspected stable runtime path did not prove automatic
   acknowledgement after host acceptance;
6. [explicit outbound publication](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/internal/extmsg/outbound.go)
   calls the adapter and returns its `PublishReceipt`. After a delivered result, Gas City
   best-effort records binding-scoped delivery context when a binding exists and appends an outbound
   transcript entry; those follow-up writes are non-fatal, and the receipt itself is not a
   separately persisted delivery proof.

Its trust and authority differ from Parler:

- adapter identity is assigned by the controller, and adapter-scoped service calls enforce the
  configured provider/account scope;
- raw in-process adapters own provider verification, while extmsg core does not authenticate the
  provider or apply adapter-scope checks to pre-normalized HTTP ingress. Stable deployments must
  protect that controller listener; optional current-main grants authorize the exact API request
  but still do not create agent-owned transcript authorship;
- Beads/controller state, not an agent signature, is authoritative for transcript authorship and
  session binding;
- the controller is the single writer, with process-local locks enforcing binding uniqueness;
- `ConversationRef` identifies an external surface, not a self-certifying agent or Gas City
  session;
- transport adapters remain thin while Gas City owns the session graph and public-speaker policy.

This is meaningful overlap with Parler. Gas City can persist and replay successfully appended,
routed external thread entries, track an explicit per-session read cursor, and issue a best-effort
normalized-path session reminder. Its hydration-pending exception is not persist-before-wake.
Parler still differs by
making independent agent keys and signed target-bound room messages the protocol boundary, by
letting agents join without becoming sessions in one orchestrator, and by binding file/code
handoffs to room-scoped content IDs with verified materialization on the visible-host path.

Current-main contains a
[connected external LLM client guide](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/docs/guides/connected-clients.md)
for registration, bearer tokens, inbound HTTP, and `Last-Event-ID` SSE replay. The pinned
[release gate](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/release-gates/ga-zy0p7n-connected-client-docs-gate.md)
explicitly calls those docs **spec-first** and says the registration/subscription endpoint
implementation has not merged. This report treats that surface as **planned**, despite its presence
in current documentation.

### Current-main API admission grants

Gas City current-main has a material security mechanism that is separate from its agent names.
When configured, typed per-city API reads and writes require Ed25519-signed, short-lived grants
from a trusted authority. Grants bind the request method, path, query/body, audience, nonce,
required city name, and, when configured, tenant CID. The default consumed-JTI guard makes them
single-use within one verifier process, but restart forgets that memory; expiry/request binding
bound the residual replay window, and a shared guard is an extension seam. Replay, audience, and
city-binding tests cover this path. See
[`writeauth.go`](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/internal/api/writeauth.go),
[`readauth.go`](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/internal/api/readauth.go),
and the [configuration reference](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/docs/reference/config.md).

The distinction from Parler remains important:

- the grant authority authorizes one API operation; an agent does not own a self-certifying
  identity through that grant;
- `gc` does not hold the authority key, but pinned current-main's
  [`grant_command`](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/cmd/gc/remote_client.go)
  flow can invoke a signer such as `gc-write-mint` and attach a fresh request-bound write grant to
  each remote mutation. Without that configured signer, a hardened endpoint needs an authenticating
  front or rejects the call; the inspected read client has no analogous grant source;
- read gating covers typed per-city routes, not the supervisor event feeds or default dashboard
  host plane on the same listener;
- the grant binds the exact API request at admission, including a mail-send body when applicable,
  but stored mail/transcript authorship is not an agent-owned signature that another reader can
  independently reverify.

This is **current-main**, not claimed for the stable `v1.3.5` dry-run. It narrows any claim that Gas
City relies only on network position, while preserving Parler's distinct message-identity model.

### Runtime providers, supervision, and recovery

Gas City's provider layer includes tmux, subprocess, exec, ACP, Kubernetes, hybrid, and other
specialized runtimes at the pinned main. Session identity, start, stop, status, and nudge belong to
the provider. The [session architecture](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/session.md)
keeps this separate from work records.

The controller follows an Erlang/OTP-inspired reconciliation model: desired agents become running
sessions; health checks observe them; restart policy is bounded by count/window; persistent work
lets a replacement process re-read its hook. The
[health-patrol design](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/health-patrol.md)
explicitly says dependency-link crash propagation is not implemented.

Gas Town adds an opinionated merge workflow. Its Refinery batches changes, tests them, and can
bisect a failing batch. Gas City maps that responsibility to configurable agents plus formulas or
post-processing rather than a universal hard-coded role.

**Decision for Parler:** do not make `ConnectorRuntime` a general process supervisor. It should
remain the trust-aware receive and host-wake boundary. Gas City or another orchestrator can own
desired state, process restarts, model/provider selection, and merge policy.

### Transport adapter as the integration seam; Pack as packaging

Gas City's shipped external-messaging `TransportAdapter` is the precise seam for a future Parler
conversation bridge. The
[Pack specification](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/docs/reference/specs/pack-spec.md)
also lets Packs compose agents, commands, services, formulas, skills, hooks, and templates through
`pack.toml` imports with pinned locks. A Pack can distribute configuration and lifecycle commands
for that adapter:

- it can be installed by users who already chose Gas City;
- it does not add Beads/Gas City concepts to Parler's beginner flow;
- version coupling, `ConversationRef` mapping, and provider-specific behavior stay outside the
  Parler wire protocol;
- it can be removed without migrating Parler state.

The repository had no GitHub release and its latest tag was `v0.3.0` at research time. A root
license was not found in the pinned snapshot, so any copied pack material would require license
clarification. An independently authored Pack that configures an independently authored adapter can
avoid that ambiguity.

### Stable binary and dry-run validation

Release archives were checksum-verified and executed from the disposable research directory:

```text
Gas City v1.3.5  darwin arm64
sha256 9468e28659c36a0c55d33e3426f932db5793fbb1abe7c922277295c457c45e44

Gas Town v1.2.1  darwin arm64
sha256 03cd4dc54939190a90b711d01c7c7ae7483337d5353e51da6005848f20e9355e
```

`gc version` returned `1.3.5`; `gt version` returned `1.2.1 (319d33a9)`. The stable Gas City CLI
exposed `gc sling --dry-run` and the documented `--nudge` separation. A disposable minimal city,
with user-global state redirected to a temporary `GC_HOME`, ran:

```text
gc sling mayor research-probe --dry-run --force --json
{"bead_id":"research-probe","dry_run":true,"method":"bead","ok":true,
 "queued":false,"routed":false,"success":true,"target":"mayor"}
```

The unavailable local Beads preflight produced warnings, as expected for the deliberately
unstarted scratch city. No routing command, runtime, session, or coding agent was launched. The
implementation's fake `SlingRunner` tests supply the deeper dispatch isolation evidence.

## Wasteland: asynchronous work federation

Wasteland is a DoltHub-backed shared work market. Rigs fork a commons database, browse work, claim
locally, submit completion evidence, and accumulate stamps/reputation. It is federation of work
records, not federation of live conversations.

An important historical/current-doc qualification comes from Gas Town's pinned
[`WASTELAND.md`](https://github.com/gastownhall/gastown/blob/67a8d72a7aa415cad5b9832bdbba31b6ec026417/docs/WASTELAND.md):
Phase 1 is “wild-west mode.” Trust levels are not enforced, claims exist first in local forks, two
rigs can claim and complete the same item, and the conflict becomes visible when histories
reconcile upstream. A GitHub PR or other external work evidence helps establish priority.

That document does not describe every current Wasteland mode. Stable `v0.4.0` and current-main both
implement direct “wild-west” mutation and PR-mode branch mutation; the stable local experiment used
the default PR mode and a per-rig/per-item branch. Stable
[`sdk/mutate.go`](https://github.com/gastownhall/wasteland/blob/5535bcdc3a368df0d4c9025dcdccc237ccf49e6e/internal/sdk/mutate.go)
routes by mode: wild-west executes on main and pushes, while PR mode executes on a branch, pushes
that branch, and may create a PR. The weak duplicate-claim/trust statement therefore applies to the
documented Phase 1 wild-west path, not as a blanket statement about all Wasteland backends today.

The stable macOS arm64 `v0.4.0` archive matched checksum
`d013bb94844da3ffd1363c30170cd7ba84722125d2af9638bbd90daf3428679f`. Its binary reported
`wl dev (commit: unknown, built: unknown)`, a minor release-provenance defect. No Wasteland remote
or account operation was attempted.

The same stable binary completed a local-only workflow in isolated XDG/Dolt directories:

1. `wl --local-db create research/commons --local-only` created the commons schema and registered a
   disposable rig.
2. `wl post ... --no-push` created wanted item `w-e7b683e559` on branch
   `wl/research-probe/w-e7b683e559`.
3. `wl claim ... --no-push` changed the branch state to `claimed`.
4. `wl done ... --evidence https://example.invalid/research-evidence --no-push` changed it to
   `in_review` and created completion record `c-f42f7e2869c035cf`.
5. An ordinary `browse` attempted upstream sync and failed because the intentionally local-only
   commons had no `upstream` remote; direct `status` still read the local branch.

The current code path matches that observation:
[`cmd_claim.go`](https://github.com/gastownhall/wasteland/blob/fbad824c1e826be00ec7e8434473e6a70556a6fd/cmd/wl/cmd_claim.go)
calls `sdk.Client.Claim`; [`sdk/mutations.go`](https://github.com/gastownhall/wasteland/blob/fbad824c1e826be00ec7e8434473e6a70556a6fd/internal/sdk/mutations.go)
builds conditional claim DML; the mode-aware mutation layer commits to main or a branch and only
pushes when enabled. Completion similarly conditionally moves the claimed item to `in_review` and
inserts evidence.

This validates the category and the mode distinction: Wasteland persists asynchronous lifecycle
and evidence in a branchable work database. It does not deliver a live turn to a running agent.

Stable `v0.4.0` and current-main support optional GPG-signed Dolt commits and `wl verify`. The
stable [`CommitSQL`](https://github.com/gastownhall/wasteland/blob/5535bcdc3a368df0d4c9025dcdccc237ccf49e6e/internal/commons/commons.go)
implements signed `DOLT_COMMIT`, and current-main's
[signing guidance](https://github.com/gastownhall/wasteland/blob/fbad824c1e826be00ec7e8434473e6a70556a6fd/README.md)
and [`cmd_verify.go`](https://github.com/gastownhall/wasteland/blob/fbad824c1e826be00ec7e8434473e6a70556a6fd/cmd/wl/cmd_verify.go)
make configured commits attributable to signing keys and their covered history tamper-evident.
Row-level author and stamp fields remain database claims rather than separately signed agent
statements. Signing was not exercised in the disposable lab. It does not create a Parler-style
self-certifying agent ID, bind a live message to a room target, or prevent two wild-west forks from
claiming the same item.

**Lessons for Parler:**

- “Federated” does not imply realtime, conflict-free, locked, trusted, or authenticated.
- If Parler later researches cross-hub federation, it must state the consistency and split-view
  model directly.
- Parler should not borrow Wasteland's work-market or reputation authority. Signed receipts may
  provide evidence to an external market, but evidence aggregation is a separate product decision.

## Parler's shipped baseline

This baseline comes from code and tests at `49a51dc`. Documentation and the backlog were used as
cross-checks, not as the source of truth. `scripts/verify.sh --rust-only` passed before drafting;
`cargo test --workspace --locked` reported 401 passing tests.

| Capability | Shipped evidence | Boundary |
|---|---|---|
| Self-certifying identity | [`Identity`](../../crates/parler-auth/src/identity.rs), hub challenge-response, signed cards | Seed stays local; no central account authority |
| Signed messages | [`MessageSig` and canonical bytes](../../crates/parler-protocol/src/hub.rs), automatic identity-backed send signing plus connector verification helpers | Receivers choose compatibility/rendering policy; hub operator still sees plaintext |
| Conversations | Portable key, owner approval option, bounded signed catch-up, presence, attachment materialization | Native visible turns only for Codex, Claude Code, and OpenCode |
| Durable delivery | SQLite room log, monotonic sequence, per-member deferred acknowledgement cursor | One hub is authoritative; push is only a latency hint |
| Retry/idempotency | `client_id`, unique `(room, author, client_id)`, reconnect-and-retry, resubscription | One append effect for the transparent retry of one `send` invocation; a new invocation or process retry is a new send |
| Role/service work | Atomic service-message claim, renewable lease, expiry/reclaim, managed worker | No task graph, planner, labels, issue CRUD, or general scheduler |
| Task lifecycle | Signed `com.parler.task` status vocabulary; managed workers emit working/terminal status and visible adapters emit terminal status | Receipt messages, not hub-enforced task state; emit behavior differs by surface |
| Memory | Private/room facts, keyed upsert, exact key, FTS5/BM25, sqlite-vec hybrid retrieval | No automatic LLM extraction or knowledge-graph consolidation |
| Artifacts | SHA-256 content IDs, upload/storage verification, room binding, file/bundle references, isolated Git bundle application | Visible-conversation downloads rehash; generic `fetch_blob`, `fetch`, and `apply` do not yet verify bytes against the requested ID |
| Discovery/A2A | Signed private-by-default cards, scoped directory tokens, A2A Agent Card projection | No inbound A2A task server or global discovery |
| Host wake | [`ConnectorRuntime`](../../crates/parler-connector/src/runtime.rs) plus three visible adapters | MCP configuration is not visible-turn parity |
| Federation | None | `KEY@HUB` is routing, not replication |

### Identity and trust

An agent ID is its Ed25519/nkey public key. The hub proves connection ownership through a
domain-separated, expiring challenge and validates any supplied self-signed card; it does not
possess the seed. Identity-backed `MeshAgent` sends sign the public-key ID, exact target,
non-signature parts, reply target, author timestamp, and UID. Receiving clients verify ordinary
message signatures, and autonomous consumers also require the signed target to match the delivered
room/DM/service. The hub still stores and relays legacy unsigned or invalidly signed records for
compatibility. Display name, role, normalized mentions, hub ID/sequence/timestamp, and the current
redacted watch-viewer projection remain hub-trusting.

This is Parler's clearest distinction from the studied ecosystem. Beads actors and assignees are
names in a database. Gas City agent names identify configured runtime roles. Wasteland identities
and stamps are work-market records. None of the inspected primary code paths established
self-certifying message authorship equivalent to Parler's. That is a negative code-search result,
not a claim that no third-party extension could add it.

### Delivery and realtime wake

The hub persists messages before fanout. A pull returns a batch and leaves its high-water pending;
`MeshAgent::commit_reads` advances the durable cursor when that batch reaches the consumer's
defined consumption boundary or local policy intentionally consumes it. Actionable visible-host
work stays unacknowledged until native acceptance or policy resolution, while terminal CLI
rendering, MCP response construction, and intentional self/invalid/replay/drop handling are other
valid boundaries. Explicit `since` reads are non-consuming. Loss triggers one transparent
reconnect/retry and subscription restoration. Within one `MeshAgent::send` invocation, reusing its
generated `client_id` makes a lost send reply return the original message. A fresh invocation,
including after an adapter restart, creates a new UID and `client_id` and is not deduplicated by
this mechanism.

This gives Parler a stronger “message arrived for this receiver” story than Gas City's nudge, while
remaining honest:

- receipt by the hub is not proof a model acted;
- cursor acknowledgement is not proof the requested work completed;
- a signed terminal `TaskRef` is the completion claim;
- push or host injection can fail while the durable log remains recoverable.

A canonical shipped path is:

1. [`MeshAgent::send`](../../crates/parler-connector/src/agent.rs) creates one `client_id`, adds the
   target-bound signature part, and sends `ClientFrame::Send`.
2. The authenticated hub path in [`server.rs`](../../crates/parler-hub/src/server.rs) checks
   membership/target limits and calls `Store::append_message`.
3. [`Store::append_message`](../../crates/parler-hub/src/store.rs) allocates the global monotonic
   message sequence, which may have gaps inside one room, and uses the unique
   `(room, author, client_id)` index to return the original row on retry.
4. Fanout announces availability; it does not replace the persisted row.
5. The receiver pulls the message, verifies the author signature and delivered-room target, and
   keeps the returned high-water pending.
6. [`ConnectorRuntime`](../../crates/parler-connector/src/runtime.rs) applies attention, replay, and
   host-wake policy. Actionable host work is committed after native acceptance; other consumer
   surfaces and intentional policy drops commit at their documented consumption boundary.

The workspace test suite, including connector E2Es and CLI/provider tests, executes
reconnect/resume, lost-reply idempotency, deferred cursor commit, half-open heartbeat recovery,
wrong-target rejection, replay rejection, and visible-host receipt paths. This is executable
workflow evidence, not a documentation-only claim.

### Tasks, roles, and the category boundary

Parler already supports useful orchestration primitives: role-addressed service rooms, atomic
claims and leases, typed handoffs, a managed worker, and signed lifecycle receipts. These are
recipes over communication. They are not a persisted project plan.

[`TaskRef`](../../crates/parler-protocol/src/hub.rs) is a status update with a required status, not
an initial dispatch envelope. Although its optional `task` accepts a client-chosen ID, shipped
visible adapters and the managed worker correlate their built-in receipts to the incoming Parler
message ID rather than preserving an external value from the request. An adapter can therefore
prototype external correlation without a wire change by putting the external reference in readable
handoff text, persisting `Parler request message ID ↔ external task reference`, and resolving the
built-in terminal receipt through that table. Accepted or working receipts are available only on
surfaces that explicitly emit them. [`Part::Extension`](../../crates/parler-protocol/src/types.rs)
preserves unknown reverse-DNS kinds if the prototype later proves that an optional machine-readable
external reference is necessary.

The authority rule should be:

> Parler says who sent the instruction or receipt, where it was delivered, and what artifact it
> referenced. Beads or another tracker says whether the underlying task exists, is blocked, is
> assigned, or is complete.

### Memory and artifacts

Parler memory is already more retrieval-oriented than Beads memory: facts have private/room scope,
keyed replacement, lexical search, exact key lookup, and optional vector ranking. Beads memory is
more operationally integrated: prime combines facts with the active work graph and host lifecycle.

The transferable improvement is not another memory database. It is better timed context:

- compact state after host context loss;
- references to retrievable facts/artifacts instead of pasted payloads;
- explicit authority and freshness in any generated brief.

Parler's content-addressed files and Git bundles are also a differentiator, with one current gap.
Beads can store external references and Gas Town can manage worktrees/merges, while Parler binds a
handoff to a room-authorized content ID. Upload and hub storage verify the SHA-256 ID, and the
visible-conversation materializer rehashes downloads. Generic `MeshAgent::fetch_blob`, `parler
fetch`, and `parler apply` do not yet compare returned bytes to the requested ID; bundle validation
only proves that the received bytes form a valid Git bundle. Parler should close that consumer-side
verification gap before claiming universal exact-byte handoff. The hub still does not review,
merge, or safely execute those bytes.

### Storage and operability

Parler's hub uses SQLite WAL with one serialized writer, a small read pool, and filesystem blob
bytes. That is a deliberate single-node operating model. It lacks Dolt's peer merge/history and
Beads' offline local database, but avoids distributed database migrations and work-state merge
conflicts in the message delivery path.

This is an appropriate trade today. The low-ops one-binary hub and small beginner flow are product
assets. Cross-hub scale should be justified by measured deployments, not by the attractiveness of
the word “federation.”

### Documentation/backlog reconciliation

The unchecked roadmap is not a reliable shipped-state ledger. Code and tests show exact keyed
recall and sqlite-vec hybrid memory are shipped. Several other unchecked items have only partial
foundations:

- `client_id` deduplicates the connector's transparent retry of one send invocation, but the
  backlog's signed-UID double-send guarantee across caller retries is not shipped;
- the request path reconnects once, resumes the cursor, and restores push subscription, but the
  queued reconnecting wrapper with exponential backoff is not shipped;
- terminal signed status messages exist, and managed workers add working status, but the backlog's
  signed request/result pair, per-service audit log, caps, hash chain, and reputation aggregation do
  not.

Other backlog capabilities remain genuinely unbuilt: tamper-evident room hash chains and split-view
detection, cross-hub federation, chunked/resumable blob transfer, full A2A task handling, code
frontier indexing, richer card offers, and receipt-derived reputation.

This drift is strategically relevant. If Parler itself cannot state what is shipped, an integration
partner cannot safely negotiate capabilities. A machine-readable capability manifest plus a
release-time doc check is more valuable now than another feature noun.

### Invariants every recommendation must preserve

1. **Seed and signing:** the seed never leaves the device; identity remains self-certifying; signed
   cards/messages remain independently verifiable.
2. **Plaintext honesty:** signatures protect identity and integrity, not confidentiality from the
   hub operator.
3. **Durable cursor truth:** push and wake remain latency paths; a cursor advances only after the
   consumer's documented boundary or an intentional local-policy disposition. Actionable or held
   host-wake work stays pending until native acceptance or policy resolution.
4. **Autonomous safety:** workspace-affecting turns require signature validity, target binding,
   replay protection, and local execution policy.
5. **Additive wire evolution:** use existing fields or `Part::Extension`; deployed clients must
   continue to render and relay unknown extensions.
6. **Scoped capabilities:** join keys, viewer tokens, and directory tokens remain distinct and
   least-privileged.
7. **Artifact safety:** blobs remain content-addressed, authorized to a room, bounded, and inert at
   the hub; every consumer path should verify downloaded bytes against the requested ID.
8. **A2A honesty:** current A2A support is discovery projection, not a task/message server.
9. **Federation honesty:** portable keys point to one hub; they do not imply replicated history.

## Adoption, field evidence, and alternatives

### Signals without false arithmetic

The following are separate snapshots retrieved on 2026-07-21 MDT / 2026-07-22 UTC. They must not
be added together: one person can star, install through Homebrew, download a release, use MCP, and
clone several times.

| Project/channel | Snapshot | What it supports |
|---|---|---|
| [Beads GitHub](https://github.com/gastownhall/beads) | 25,485 stars; 1,717 forks | High developer awareness, not active use |
| [Gas Town GitHub](https://github.com/gastownhall/gastown) | 17,148 stars; 1,576 forks | Strong interest in orchestration, not deployment scale |
| [Gas City GitHub](https://github.com/gastownhall/gascity) | 1,017 stars; 332 forks | Early but material platform interest |
| [Packs GitHub](https://github.com/gastownhall/gascity-packs) | 57 stars; 47 forks | Nascent ecosystem; fork ratio may reflect customization |
| [Wasteland GitHub](https://github.com/gastownhall/wasteland) | 83 stars; 28 forks | Experimental federation interest |
| [Beads Homebrew](https://formulae.brew.sh/formula/beads) | 1,978 installs / 1,637 on-request in 30 days; 18,867 / 15,939 in 365 days | Strongest public install signal, still not unique users |
| [Gas Town Homebrew](https://formulae.brew.sh/formula/gastown) | 155 installs in 30 days; 4,533 in 365 days | Smaller install footprint; formula lagged at 1.1.0 while GitHub stable was 1.2.1 |

No credible public source established daily active users, retention, production deployments,
simultaneous agents, Wasteland federation nodes, or paid adoption. Founder scale claims and stars
are not substitutes. This report leaves those metrics **unverified**.

### What adoption evidence does say

1. **Agent-first local tooling attracted substantial public attention and install volume.** Beads'
   compact `create → ready → claim → close` loop, offline behavior, and host integration offer a
   plausible product explanation, but a dated snapshot cannot establish growth rate or causation.
2. **Gas Town shows high awareness alongside a smaller single-channel install signal.** Its star
   count is large and its Homebrew count is smaller than Beads'. The populations, time windows, and
   product ages differ, so no awareness-to-adoption or cross-product conversion conclusion follows.
3. **Distribution drift creates support cost.** At the snapshot, GitHub, Homebrew, and
   [PyPI `beads-mcp`](https://pypi.org/project/beads-mcp/) were aligned at 1.1.0, while Gas Town's
   Homebrew formula lagged its GitHub release. An integration must negotiate capability/version
   rather than infer it from a package name.
4. **Human control surfaces are partly externalized.** The stable repository's curated
   [community-tools list](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/docs/COMMUNITY_TOOLS.md)
   names third-party web/Kanban apps, IDE integrations, and orchestration/messaging wrappers. That
   proves ecosystem experimentation, not usage, quality, or continued maintenance. Core authority
   remains in CLI/database/control-plane paths.

### Praise, complaints, and failure evidence

The strongest positive evidence is behavioral, not testimonial: the maintained community-tools
catalog and Gas City Packs tree contain dashboards, tracker integrations, orchestration layers,
and reusable packs around the structured graph. The first-party repositories' large test and
migration surfaces show sustained maintainer investment. This research did not find a defensible
sample from which to claim recurring independent praise, retention, or production success.

The strongest negative evidence is first-party engineering history:

- the `v1.1.0` release had to repair remote migration and schema divergence;
- current-main adds guarded writes because read-then-write checks were insufficient under
  concurrency;
- initialization/destructive-operation hardening followed real data-loss risk;
- maintainer policy was tightened after an audit found defects correlated with bypassed review;
- stable embedded `bd sql` help/behavior disagreed in the hands-on test.

An [independent Hacker News comment](https://news.ycombinator.com/item?id=46458936) described
overlapping concepts, bugs, and documentation as overwhelming. That is one anecdote, not a
population study. It is consistent with the objectively large ontology and the migration/release
history, but it should not be inflated into recurring user sentiment or a market-wide conclusion.
The maintainer/community
[UI discussion](https://github.com/gastownhall/beads/discussions/276) independently shows the
intentional CLI-first stance and churn among UI attempts, again without establishing prevalence.

### Adjacent and competing approaches

| Category | Examples | Relative strength | Relative gap that Parler can address |
|---|---|---|---|
| Human issue trackers | GitHub Issues, Jira, Linear | Human UI, organization workflow, integrations | Agent-native live signed conversation |
| Local task ledgers | Beads, Taskwarrior-style tools, smaller Beads alternatives | Offline structured work and ready queues | Cross-host identity/delivery |
| Planning/spec systems | Spec Kit, OpenSpec, BMAD, Superpowers, gstack | Decomposition, review rituals, intent capture | Realtime independent-agent coordination |
| Agent orchestrators | Gas Town, Gas City, Beads-based supervisors | Runtime lifecycle, scheduling, recovery, merge policy | Transport trust and portable conversation |
| Human chat | Slack, Discord, Teams | Mature people UX and notifications | Self-certifying agent identity, typed artifacts, agent-visible turns |
| Interop protocols | MCP, A2A, ACP | Tools/resources, agent cards/tasks, runtime/session abstractions | A durable signed room with receiver cursors |
| Agent memory | Project instructions, vector stores, knowledge graphs | Rich recall or semantic consolidation | Scoped shared facts tied to authenticated conversation |

The product opportunity is not “one system that replaces all rows.” It is a small communication
primitive that composes with them.

## Comparison matrix

The matrix compares shipped/current behavior at the pinned references. “No” means no verified core
implementation was found, not that a third-party adapter is impossible.

| Dimension | Beads | Gas Town / Gas City | Wasteland | Parler |
|---|---|---|---|---|
| **Identity and trust** | Actor/assignee strings; no cryptographic author principal found | Configured names/roles/providers; current-main optionally verifies authority-issued API grants, not agent-owned message identity | Registered rigs/stamps; optional GPG commit attribution; wild-west trust levels not enforced | Ed25519 public key is agent ID; challenge-response; signed cards/messages |
| **Primary persistence** | Local Dolt database; JSONL interchange | Beads work/default `beadmail`/extmsg transcripts and memberships plus control/event/runtime state; `exec` mail is provider-defined | DoltHub-backed sovereign forks | Hub SQLite room log/facts plus filesystem blobs |
| **Task graph** | First-class issues, dependencies, blockers, ready views, labels, comments | Queries and formulas consume/control Beads graphs | Federated work records and claim evidence | No project task graph; typed communication receipts only |
| **Transport** | Explicit database push/pull/remotes; `bd mail` delegates | Store/runtime/provider calls, extmsg transport adapters and typed HTTP API, external commands/events | Fork/push/pull/reconcile | WebSocket hub with CLI/MCP and REST projections |
| **Delivery guarantee** | Durable database mutation; not a network-delivery acknowledgement | Default beadmail durable, exec mail provider-defined; nudge lossy; successfully appended routed extmsg entries have sequence, provider-ID dedupe, membership replay/explicit ack; hydration-pending may notify without append; outbound returns a publish receipt but post-delivery record writes are non-fatal | Eventual database reconciliation; conflicts possible | Persist-before-fanout, durable per-member cursor, explicit ack; one send invocation's transparent retry is append-idempotent |
| **Realtime wake** | None in core | Provider nudge is lossy; normalized extmsg ingress issues best-effort reminders; successfully appended routed entries are backfillable, but hydration-pending may notify without an entry | None verified | Push/long-poll doorbell plus host adapter; durable log remains recovery |
| **Visible-host support** | Instructions/hooks/plugins prime hosts; not shared visible chat | Provider-controlled sessions; extmsg reminds/wakes those sessions, not a portable independent-host room | CLI/database workflow | Native conversation turns in Codex, Claude Code, OpenCode; MCP elsewhere |
| **Orchestration** | Models workflow state but charter excludes scheduling/runtime policy | Core purpose: dispatch, processes, supervision, recovery, formulas, merge flow | Work-market coordination, not local runtime orchestration | Role claims/worker recipes, not a general controller |
| **Memory** | Operational graph, decisions, comments, `remember`, `prime`, compaction | Persistent work/default mail plus extmsg shared transcripts and session logs support recovery/replay | Work history/evidence/reputation records | Scoped keyed facts, FTS5/BM25, sqlite-vec, conversation/artifact history |
| **Artifacts/code** | External refs and structured records | Worktrees, rigs, merge strategies/Refinery | PR or external evidence establishes delivered work | Content-addressed files and Git bundles bound to rooms; generic download rehash gap remains |
| **Sync/federation** | Dolt remotes and stable/current peer sync; schema/data merges | Delegates work sync to Beads/Dolt and providers | Explicit asynchronous work federation | Single authoritative hub; no cross-hub federation |
| **Conflicts/concurrency** | Transactions, conditional claims, Dolt cell merge, CAS/row lock on main | Durable route plus runtime recovery; not exactly-once execution | Wild-west can reconcile duplicate claims later; PR mode isolates/reviews branches | Atomic service claim/lease; global monotonic message sequence plus room cursors; replay guard for execution |
| **Security/confidentiality** | Plaintext; no database ACL/encryption in core | Local/store/provider controls plus optional current-main signed per-city API grants; some listener surfaces remain outside read gate | Depends on DoltHub/fork governance; optional signed commits, row authors remain claims, no duplicate wild-west claim prevention | Self-certifying identity and target-bound message integrity; scoped capabilities; hub sees plaintext |
| **Operability** | Excellent offline/local query; substantial Dolt bootstrap/migration/version surface | Powerful but high concept/process/provider load | Requires remote account/fork/reconciliation operations | One Rust binary and low-ops hub; single-node scaling ceiling |
| **Extensibility** | Metadata-first, hooks, tracker adapters, CLI/MCP, formulas | Packs, runtime/mail/event/extmsg adapters, services, hooks, formulas/orders | Backend/client integrations around common records | `MeshTransport`, reverse-DNS parts, host adapters, CLI/MCP |
| **Governance** | MIT; explicit contributor/review policy; active migration hardening | MIT in main projects; platform is evolving quickly | MIT; phase semantics still evolving | Apache-2.0; explicit engineering/review/security invariants |
| **UX complexity** | Small happy path, large advanced ontology and storage modes | Large ontology is intrinsic to orchestration | Account, fork, claim, evidence, sync concepts | Small beginner conversation flow; advanced surfaces hidden, but MCP/host parity can confuse |

### Architectural conclusions supported by traces

1. **Beads is a work ledger:** the stable create trace ends in a Dolt transaction/commit, and the
   ready/claim/close lab manipulates graph state without a delivery path.
2. **Gas City is a control plane with an external messaging fabric:** sling resolves persistent
   work, invokes an injectable routing runner, then optionally nudges a provider. Extmsg maps
   provider conversations into controller-owned sessions and Beads-backed membership/transcripts.
3. **Wasteland is asynchronous work federation:** the local workflow produced branch-scoped
   claim/evidence state; wild-west docs defer duplicate claims to reconciliation, while current PR
   mode adds branch review instead of live delivery.
4. **Parler is a communication plane:** send persists a signed target-bound message; receive
   advances a member cursor at the consumer's defined boundary or an intentional policy
   disposition; a host adapter may turn actionable input into a visible model turn.

These conclusions rely on at least one code path and one executed local workflow for all four
categories. The Wasteland workflow remained local-only because external account and remote
mutation were explicitly out of scope.

## Decisions: borrow, integrate, differentiate, avoid

### Borrow

These patterns improve Parler without changing its category.

1. **Name the authority for every state.** Beads distinguishes durable issue state, local wisps,
   and node-local leases. Gas City distinguishes mail, nudge, and events. Parler should document,
   for every integration event, whether the authoritative fact is the room log, receive cursor,
   external task record, runtime process, or receipt.
2. **Reference external work instead of mirroring it.** Beads' metadata-first charter is a useful
   restraint. Readable external references plus adapter correlation are safer than a Parler task
   schema that drifts from Beads, GitHub, Linear, or Jira.
3. **Keep liveness out of durable history.** Beads moved lease heartbeats to ignored local tables.
   Parler already treats push as a doorbell. Continue that design for presence, wake retries, and
   adapter health; do not inflate the signed message log with heartbeat noise.
4. **Use conditional writes for ownership transitions.** Beads' claim/CAS work and Parler's service
   claims share the right pattern: one transaction establishes one winner; re-claiming the same
   known item by its owner is idempotent; expiry and reclaim are explicit. Do not describe fresh
   queue acquisition as request-idempotent.
5. **Treat migrations and capability skew as distributed product problems.** Beads' single-migrator
   controls are specific to Dolt, but the wider lesson applies. Parler integrations need explicit
   versions/capabilities, forward-compatible unknown extensions, and refusal rather than ambiguous
   downgrade.
6. **Inject current context at the moment it is useful.** `bd prime` and post-compaction
   reinjection are more efficient than permanently expanding the prompt. Parler should assemble a
   compact conversation/task-reference brief only when a host starts, reconnects, or compacts.
7. **Design post-commit hooks and fakeable side effects.** Gas City's injectable `SlingRunner` and
   Beads' after-commit hooks make failure boundaries testable. Any Parler adapter should separate
   durable append, external tracker write, and host wake behind injected seams.
8. **Preserve provenance under agent-generated change volume.** Beads' governance scars support
   Parler's existing small-diff, negative-test, attribution, and review discipline.

### Integrate

These belong in optional adapters or extensions:

1. **Beads task-reference adapter:** put an opaque issue reference in readable addressed-handoff
   text, persist the Parler-request-to-task mapping, and resolve built-in receipts through it;
   optionally render title/status fetched from Beads at the edge.
2. **Receipt-to-ledger updater:** with explicit user enablement, translate only the lifecycle
   receipts a selected surface actually emits into Beads comments/status using conditional writes.
   Beads remains authoritative and failed writes stay visible for retry.
3. **Gas City external-messaging adapter:** map a Parler room/conversation to Gas City's
   `ConversationRef` through its shipped transport-adapter seam. Gas City retains its transcript,
   session ingress, runtime providers, formulas, and supervision; a Pack can package/configure the
   adapter.
4. **Artifact evidence bridge:** attach Parler file/bundle content IDs to external work records.
   The tracker records the reference; Parler authorizes the room-scoped bytes, and every bridge
   consumer must rehash them before claiming exact-byte evidence.
5. **Event observation adapter:** project selected Parler delivery/receipt events into Gas City
   observation streams without treating best-effort event recording as command delivery.

None requires a new beginner command or core database table.

### Differentiate

Parler should own and prove these capabilities more clearly:

1. **Self-certifying agent identity.** A name is presentation; the key proves the sender.
2. **Signed target-bound conversation.** A relay cannot forge a message that a verifying client
   accepts as authored by a given public key, nor change its signed target, reply link, or typed
   parts. Display metadata and the current watch-viewer projection remain hub-trusting.
3. **Receiver-controlled durable state.** Per-member acknowledgement cursors survive reconnect;
   push is only a latency hint, and each consumer documents when it commits.
4. **Portable live conversation across independent hosts.** A private join command gets another
   supported agent into the same caught-up conversation.
5. **Visible turns with an explicit host contract.** Parler can wake supported visible interfaces
   without pretending every MCP host has that ability.
6. **Trust-aware autonomous handoff.** Signature, target binding, replay protection, local policy,
   correlated task receipt, and a verified artifact reference form one auditable path. Signer
   authenticity is evidence; adapter authorization remains an explicit policy decision.
7. **Low-ops composition.** One binary can be a CLI, MCP server, connector, or hub without forcing
   users to adopt a work database or orchestration ontology.

### Avoid

These lessons are valuable chiefly as warnings.

1. **Do not put Dolt in the message/cursor path.** It would add database-history merge, bootstrap,
   remote credentials, designated migrations, and operator conflict resolution to a problem with a
   simpler authoritative-log solution.
2. **Do not add first-class issue CRUD or dependency readiness.** It creates a second source of
   truth and puts Parler in direct competition with Beads and human trackers.
3. **Do not embed formulas, molecules, gates, or swarm semantics in protocol types.** Their meaning
   belongs to an orchestrator; unknown extension parts plus human-readable text are sufficient for
   transport.
4. **Do not turn host wake into universal runtime supervision.** Process desired state, restart
   budgets, provider login, model selection, worktrees, and merge queues belong above Parler.
5. **Do not build a work marketplace or reputation score before receipt evidence is robust.**
   Wasteland shows how quickly federation language outruns lock/trust guarantees.
6. **Do not mirror provider-specific role names in core.** Mayor, Polecat, Refinery, tmux, and
   Kubernetes are integration configuration, not durable wire vocabulary.
7. **Do not make setup silently invasive.** Any host hook or config edit must remain previewable,
   scoped, merge-preserving, and reversible.
8. **Do not report telemetry or capability claims that code cannot prove.** The Beads security-page
   mismatch and Parler backlog drift are the same class of trust erosion.

## Interoperability choices

| Choice | Category fit | Value | Coupling/risk | Effort | Decision |
|---|---|---|---|---|---|
| **A. No integration** | Safe but passive | Preserves focus; users manually paste task IDs | Misses a credible complementary workflow and leaves positioning abstract | S | Keep as fallback, not the strategy |
| **B. Beads task-reference adapter** | Excellent | Signed addressed handoff plus correlated receipt/artifact evidence around an external task authority | Authorization, version discovery, restart correlation, and conditional writeback must be correct | M | **Prototype now** |
| **C. Gas City external-messaging bridge** | Good if external | Parler-authenticated room input reaches Gas-controlled sessions through the shipped extmsg seam | Two durable transcripts, trust translation, restart idempotency, event loops, and autonomous safety | M/L | Prove after B; ship only with usage evidence |

### Recommended option B: task-reference adapter

This is an additive proposal, not an implementation specification.

The smallest useful flow is:

1. A dispatcher selects an existing Beads issue. The adapter reads its ID, current version,
   assignment/blocker state, and enough display data to make the Parler message understandable. It
   creates a durable outbox/correlation record before sending.
2. Parler sends a normal signed message containing human-readable instructions, the readable
   namespaced external reference, and an explicitly addressed `HandoffRef`. A DM or atomic
   role/service claim is also valid. The initial request does not carry `TaskRef`, because that type
   is a lifecycle status update and has no requested/dispatched state.
3. The receiver's normal Parler cursor, signature verification, target binding, replay guard, and
   local policy apply unchanged.
4. Shipped visible adapters return a signed terminal receipt whose `TaskRef.task` and signed
   `replyTo` point to the incoming Parler request ID. Managed-worker paths may also emit working
   status. The adapter resolves that request ID through its correlation table; accepted/working or
   awaiting updates exist only where a surface explicitly emits them.
5. Results may point to a Parler content-addressed blob or bundle. The adapter rehashes downloaded
   bytes against the requested ID before treating them as evidence.
6. An optional adapter writes selected state/comments back to Beads only after re-reading the
   current issue version, ownership/assignee, blockers, and allowed transition. A conflict is
   surfaced; it is never hidden behind last-write-wins behavior.

Authority rules:

- Beads owns existence, title, blockers, readiness, assignee, and final tracker status.
- Parler owns public-key authorship proof for signed fields, the room record/cursor, receipt
  signature, and room-scoped blob access. Display metadata remains hub-trusting.
- The adapter owns its outbox, correlation, authorization, and retry bookkeeping, not either
  source's domain data.
- A valid signature proves which key made a claim; it does not authorize that key to mutate an
  arbitrary Beads issue. The adapter requires an expected public-key/assignee allowlist, addressed
  dispatch, signed `replyTo` correlation, and an allowed tracker transition.
- A terminal Parler receipt is evidence, not permission to override a human-edited or blocked Beads
  record.
- Unknown/unavailable Beads versions degrade to an opaque reference plus readable text; they do not
  make the message undecodable.

The prototype should avoid a new extension kind initially. Readable text plus persistent
`Parler request ID ↔ external task ID` correlation works with deployed clients. Although
`TaskRef.task` permits a client-chosen ID for custom clients, built-in receipts overwrite it with
the incoming Parler message ID. Only repeated missing semantics could justify an optional
namespaced `Part::Extension`; a standard new type would require stronger evidence.

The shipped CLI/MCP surface does not expose arbitrary composition of readable text plus
`HandoffRef` plus custom extension fields in one call. The prototype therefore needs a small client
using the connector API; this is an integration-package cost, not a reason to enlarge the public
wire protocol.

There is also an honest crash window: `MeshAgent::send` generates a new UID and `client_id` for
each invocation. Its internal reconnect retry is idempotent, but a fresh adapter invocation after a
crash is not. The prototype must persist outbox state, reconcile ambiguous sends, and measure the
send-success-before-correlation-recorded case. If real bridges need caller-stable retry, an
additive connector API could expose the existing wire `client_id`; the wire frame need not change.

### Conditional option C: Gas City bridge

If option B produces repeat usage, implement Gas City's stable external-messaging
`TransportAdapter`, optionally packaged by a Pack:

```text
signed Parler room log
        |
        | verify signature, target, membership, sender policy
        v
durable bridge outbox/correlation -> Gas City extmsg TransportAdapter
                                      |
                                      v
                         extmsg transcript + membership
                                      |
                                      | session reminder / provider wake
                                      v
                            Gas-City-owned session
                                      |
                                      | explicit extmsg publication
                                      v
Parler room <- adapter publish + bridge signature/correlation
```

Gas City's extmsg transcript is authoritative for its controlled session's external-conversation
membership and read position. The Parler room log is authoritative for Parler members' signed
messages and receive cursors. The bridge correlates records; it does not mirror either transcript
as a bidirectional last-write-wins database. Gas City's provider/controller, not Parler's native
visible adapter, wakes a Gas-owned session. A separately Parler-owned host session can use Parler's
native adapter, but Gas City then does not own that runtime.

On the outbound path, a bridge identity signs as the bridge. It must not present a Gas City worker
as the cryptographic author. Preserving worker-level Parler authorship requires that worker to own
and use its own Parler identity/connector.

Required safeguards:

- require a successful extmsg transcript append before treating a reminder as executable input.
  Stable normalized ingress can still notify on `ErrHydrationPending` without an entry, so an
  experimental bridge must either suppress/retry that path through an upstream seam or fail the
  persist-before-wake go criterion;
- a durable bridge outbox records processed source events, provider message IDs, and correlations;
  Parler's current per-invocation send ID alone does not deduplicate a restarted bridge;
- a bridge-origin marker prevents Parler → Gas City → Parler loops;
- nudge failure never deletes or closes durable work;
- only verified, target-bound messages from an authorized sender may enter Gas City; provider-ID
  dedupe and persistent correlation reject replays before session ingress;
- the Gas City provider/controller remains the authority on whether a session should exist;
- a crash before admission may retry, and an explicit pre-action rejection may be released; a crash
  after Parler's persistent autonomous replay admission is fail-closed and requires visible
  recovery/re-dispatch rather than silent automatic reinjection;
- event projection is explicitly best-effort and cannot stand in for a receipt;
- credentials and Parler seeds remain separate; the bridge never exports a seed into Pack config.

A runtime-provider integration is premature. The shipped transport-adapter seam validates demand
with less ownership ambiguity and can be removed cleanly; a Pack is useful packaging, not the wake
or transport mechanism itself.

## Positioning

### Category statement

> **Parler is the authenticated communication plane for independent AI agents: self-certifying
> identity, signed target-bound messages, live rooms, durable receiver cursors, and host-visible
> turns. It complements task ledgers and orchestrators.**

The short contrast is:

> Beads remembers and coordinates the work. Gas City routes external conversations into sessions
> it controls. Parler gives independent agents verifiable public-key authorship and a portable
> signed room across hosts.

### Proof points

- The private key stays on the agent's device; the public key is the identity.
- Messages are signed over their exact target and typed parts.
- A relay cannot forge a message that a verifying client accepts for a public key, although it can
  read plaintext and controls unsigned display metadata and the current watch-viewer projection.
- Per-member cursors survive disconnect and advance at each consumer's documented boundary;
  actionable visible-host work waits for native acceptance or policy resolution.
- One portable invitation catches another agent up in the same live conversation.
- Codex, Claude Code, and OpenCode can receive normal visible turns; other configured hosts receive
  MCP tools without a false parity claim.
- Files and code bundles are authorized to a room by content ID; upload/storage and visible-host
  materialization verify the hash, while generic download verification is a named gap to close.
- Typed handoffs and message-ID-correlated receipts compose with external work authorities without
  requiring a task database in Parler.

### Likely sources of confusion

1. **“Messaging”** may sound like Beads mail, Gas City nudge, Slack chat, or a protocol with durable
   acknowledgement. Lead with the latter and explain the other three.
2. **“Multi-agent”** may imply Parler launches and schedules agents. It does not; it connects
   independent agents and offers bounded role-queue recipes.
3. **“Memory”** may imply a task graph or automatic cognitive memory. Parler stores scoped,
   retrievable facts and conversation/artifact history.
4. **“Federation”** may be inferred from portable keys or A2A cards. Neither is cross-hub state
   replication.
5. **“Private”** may be interpreted as end-to-end encrypted. Private means scoped membership and
   access; the hub sees plaintext.
6. **“Host support”** may conflate MCP configuration with visible-turn injection. Publish both
   matrices together.

### Claims Parler should not make

- end-to-end confidentiality from the hub operator;
- universal visible wake across all MCP hosts;
- exactly-once task execution or proof that a model acted;
- a first-class issue graph, workflow planner, or process supervisor;
- cross-hub federation, global discovery, or split-view resistance today;
- full A2A task/message interoperability;
- trustless reputation from self-reported tokens, time, or completion;
- safe execution or semantic review of transferred code.

Honest boundaries strengthen the differentiated claims. They do not weaken them.

## Ranked improvement roadmap

### Ranking gates

Every ranked item passed these gates:

1. **Protocol fit:** it strengthens authenticated communication, delivery, wake, or evidence rather
   than making Parler a work ledger or orchestrator.
2. **Additive compatibility:** existing clients can continue to connect, render messages, and use
   rooms; adapters and existing fields come before new wire semantics.
3. **Authority clarity:** the proposal names Parler, tracker, controller, runtime, and operator
   sources of truth without bidirectional last-write-wins mirroring.
4. **Security preservation:** seed locality, message signing, target binding, replay protection,
   scoped capabilities, plaintext honesty, and blob bounds remain intact.
5. **Evidence threshold:** an observed user/problem signal or a falsifiable prototype justifies the
   work.
6. **Operational boundedness:** a failure has a visible recovery path and does not require Parler's
   beginner to learn Beads/Gas City.

Items that fail a gate remain in the **Avoid** bucket even if they are technically interesting.

### Now: 0–90 days

#### N1. Publish one capability matrix/manifest and category boundary

- **Outcome:** one maintained capability matrix states shipped, host-specific, experimental, and
  unbuilt behavior; docs explain “communication plane” and show Beads/Gas City as complements.
  Extend or generate from the shipped `hub_capabilities` source already projected by `/api/hub`
  and `/.well-known/parler.json` instead of creating a parallel registry.
- **Evidence:** Parler's backlog lacks granularity: keyed recall and sqlite-vec are shipped, while
  broad retry/reconnect and receipt items have only partial foundations. Beads' telemetry/security
  and package-version drift show the trust cost of contradictory surfaces.
- **Product layer:** documentation, release tooling, CLI diagnostics; no protocol change.
- **Dependencies:** inventory the existing CLI/MCP/host registries and code-backed tests.
- **Effort:** **S/M**.
- **Compatibility/security constraints:** never infer a capability solely from version; keep A2A,
  federation, privacy, and visible-host qualifications explicit.
- **Success metric:** release CI detects a deliberately introduced capability/doc mismatch; all
  existing stale claims named in this report are resolved; five fresh evaluators can correctly
  distinguish Parler, Beads, and Gas City after reading the first page.
- **Stop/go criterion:** **Go** to integration docs only when the manifest can be generated or
  checked without creating a second manual source of truth. **Stop** a machine manifest if it
  becomes another hand-maintained registry.

#### N2. Prototype the Beads task-reference adapter outside core

- **Outcome:** demonstrate addressed dispatch, visible handoff, built-in terminal receipt
  correlation, verified artifact return, and optional conditional Beads writeback around one
  external issue. Treat accepted/working status as optional and surface-specific.
- **Evidence:** `HandoffRef`, signed messages, message-ID-correlated built-in receipts, and blobs
  cover the transport when paired with adapter state; the stable Beads lab proves
  ID/readiness/claim/close semantics; the category fit is complementary.
- **Product layer:** example/integration repository or optional adapter package, plus an operator
  guide. No new frame, schema, beginner command, or MCP tool.
- **Dependencies:** stable Beads CLI capability detection; a readable namespaced-reference
  convention; durable outbox and `Parler request ID ↔ external task ID` correlation; N1
  terminology.
- **Effort:** **M**.
- **Compatibility/security constraints:** invoke Beads with telemetry choice under user control;
  never read/export Parler seeds; require signature/target/replay checks, an addressed handoff,
  signed `replyTo`, an expected signer/assignee allowlist, and an allowed tracker transition. Recheck
  issue ownership, blockers, and version before guarded writeback; surface conflicts.
- **Success metric:** two disposable clones complete the end-to-end scenario; a built-in terminal
  receipt resolves through the correlation table; reprocessing one receipt causes one tracker
  transition; a validly signed unauthorized receipt and a substituted task reference are rejected;
  the send-success-before-correlation crash window is reconciled or explicitly reported as
  ambiguous; a same-field tracker conflict remains unresolved and visible; deployed clients render
  the exchange without upgrade.
- **Stop/go criterion:** **Go** to a supported kit after at least three independent users repeat the
  workflow and two use it again within 30 days. **Stop** if the adapter must mirror the Beads graph
  or add mandatory wire fields to be useful.

#### N3. Add an integration delivery/wake conformance harness

- **Outcome:** a reusable test harness proves persist-before-wake, documented cursor boundaries,
  one-invocation retry idempotency, target binding, replay rejection, durable adapter correlation,
  and visible-host failure handling.
- **Evidence:** Gas City deliberately separates durable route from lossy nudge; Wasteland exposes
  eventual conflicts; Beads uses fakeable runners and backend conformance tests. Parler's
  integration claim needs the same executable boundary.
- **Product layer:** connector/CLI test infrastructure and adapter SDK fixtures.
- **Dependencies:** existing in-process hub E2E tests, `MeshTransport`, `ConnectorRuntime`, and fake
  host injectors.
- **Effort:** **M**.
- **Compatibility/security constraints:** include negative tests for unsigned, wrong-target,
  replayed, unauthorized-but-validly-signed, expired-lease, substituted-reference, and duplicate
  bridge events; never treat a successful wake as durable receipt. Test crash points before send,
  after send but before correlation, before replay admission, and after persistent replay
  admission; the last is fail-closed and operator-visible rather than automatically reinjected.
- **Success metric:** CI injects a dropped WebSocket reply, lost wake, host rejection, duplicate
  event, and tracker conflict. A dropped reply during one send invocation yields one append; a
  duplicate source-event ID yields one adapter transition through persistent bridge state. Each
  case preserves the durable message/cursor and exposes any ambiguous crash window; model and
  external side effects remain outside the guarantee.
- **Stop/go criterion:** **Go** before supporting any external writeback or Gas City bridge.
  **Stop** an adapter release that cannot pass the common harness.

#### N4. Measure and simplify the advanced context/tool surface

- **Outcome:** retain the three-step beginner flow while offering a compact advanced profile or
  capability discovery path that avoids loading unrelated orchestration, memory, artifact, and
  directory concepts into every agent session.
- **Evidence:** Beads' lazy MCP discovery and `prime` show the value of timely context; one
  independent complaint plus the objectively large ontology motivates measurement, not a claim of
  population-wide correlation. Parler has many advanced CLI/MCP surfaces hidden from beginner docs
  but still visible to configured agents.
- **Product layer:** MCP presentation, prompts/resources, and host-start context; protocol remains
  unchanged.
- **Dependencies:** measure actual tool-definition bytes and per-tool usage before choosing a
  profile boundary.
- **Effort:** **M**.
- **Compatibility/security constraints:** legacy clients retain the full surface; a compact profile
  must not hide safety warnings or silently change tool behavior.
- **Success metric:** establish a checked baseline, then reduce the default advanced schema/context
  bytes by at least 30 percent for conversation-only sessions with zero task-success regression in
  a representative evaluation.
- **Stop/go criterion:** **Go** only if measured usage is concentrated enough to define a stable
  core. **Stop** profile proliferation if users must understand more modes than the bytes save.

#### N5. Close the generic blob-download verification gap

- **Outcome:** every public blob retrieval path verifies `SHA-256(bytes) == requested content ID`
  before materialization, Git bundle validation, or adapter evidence use.
- **Evidence:** upload/storage and visible-conversation materialization already verify content, but
  `MeshAgent::fetch_blob`, `parler fetch`, and `parler apply` do not currently bind returned bytes
  to the requested ID. A malicious or faulty hub can therefore substitute bytes on those paths.
- **Product layer:** connector and CLI validation plus focused negative tests; no wire change.
- **Dependencies:** existing content-ID helper, blob fixtures, and error-reporting conventions.
- **Effort:** **S**.
- **Compatibility/security constraints:** reject mismatches before writing or invoking Git; retain
  room authorization, size bounds, and inert hub storage; do not describe bundle syntax validation
  as content-ID verification.
- **Success metric:** a fake transport that returns valid bytes under the wrong requested ID is
  rejected by connector, fetch, apply, and adapter-facing paths; valid existing downloads remain
  compatible.
- **Stop/go criterion:** **Go** immediately because the content-addressed interface already promises
  this binding. **Stop** release of any new artifact consumer that bypasses the common verifier.

### Next: 3–6 months

#### X1. Turn the successful task-reference prototype into a supported adapter kit

- **Outcome:** a documented adapter contract, capability negotiation, correlation store, retry
  policy, and examples for Beads first and one human tracker second. The second tracker proves the
  abstraction is not Beads-shaped.
- **Evidence:** only N2 repeat usage can justify support; Beads' mode/version drift and current-main
  guarded writes show why capability checks and conditional updates are mandatory.
- **Product layer:** optional SDK/examples and integration documentation; perhaps a separately
  versioned executable.
- **Dependencies:** N1, N2 go criterion, N3 harness, stable opaque-reference convention.
- **Effort:** **M**.
- **Compatibility/security constraints:** no tracker credential enters the hub; no external task
  content becomes public through directory cards; unknown versions degrade read-only; writeback is
  opt-in and auditable.
- **Success metric:** two tracker adapters pass the same conformance suite; at least 95 percent of
  pilot correlations complete without manual relinking; every failed writeback has a visible,
  replay-safe retry path.
- **Stop/go criterion:** **Go** when the second tracker uses the same authority/correlation model
  without schema branching. **Stop** generalization and keep a Beads-specific example if the common
  layer becomes a lowest-common-denominator task API.

#### X2. Add tamper-evident room-history verification

- **Outcome:** detect deletion or reordering relative to observed signed checkpoints, and detect
  split views when participants compare checkpoints through an independent witness/channel. An
  isolated client cannot detect a suffix it never saw; the UI and threat model must say so.
- **Evidence:** Parler already differentiates on sender authenticity, but its current hub can omit
  signed messages or present different histories. Neither Beads/Dolt history nor Parler's existing
  message signatures solve the receiver-consensus problem.
- **Product layer:** protocol/store/connector security, designed additively and already represented
  as an unbuilt backlog direction.
- **Dependencies:** threat model, chain/checkpoint design, old-client behavior, retention semantics,
  multi-device verification model.
- **Effort:** **L**.
- **Compatibility/security constraints:** never describe a hub-authored hash as proof against that
  hub. Do not simply add `prev` to the existing `com.parler.sig` signed bytes: old clients would
  ignore the new field, reconstruct the old canonical payload, and reject the new signature. Use
  either a separate chain/checkpoint extension covered as a normal non-signature part by the valid
  v1 signature, or dual v1/v2 signatures. Preserve cursor/retention semantics and bind checkpoints
  to self-certifying participants or make the remaining trust explicit.
- **Success metric:** fault-injection tests detect a deletion/reorder against a previously observed
  checkpoint, invalid checkpoints, and equivocation after independent checkpoint comparison. A
  deployed old client continues to decode the room **and verify the v1 message signature**;
  documentation explicitly states the unseen-suffix and no-independent-witness limits.
- **Stop/go criterion:** **Go** only after an independent security review agrees the design adds
  evidence rather than security theater. **Stop** if the proposal requires a central trusted signer
  or a breaking history rewrite.

#### X3. Prove a Gas City external-messaging transport adapter

- **Outcome:** map one Parler room to a Gas City `ConversationRef`; admit authorized signed room
  input into the extmsg transcript, let Gas City's provider/controller wake its own session, and
  publish correlated output back to Parler without making the bridge a scheduler.
- **Evidence:** stable Gas City already ships the `TransportAdapter`, transcript, membership,
  dedupe, explicit-ack, and session-reminder seams. This is more precise than attaching a generic
  sidecar after sling or claiming a Parler native adapter can wake a Gas-owned session. However,
  normalized hydration-pending ingress may notify without a transcript entry, which the prototype
  must resolve rather than hide.
- **Product layer:** externally versioned extmsg adapter, with an optional Pack for installation and
  configuration; not Parler core or a Gas City runtime provider.
- **Dependencies:** X1 adapter contract, N3 harness, stable extmsg API across two releases, a Gas
  City compatibility matrix, durable bridge outbox/correlation, a persist-before-wake resolution
  for hydration-pending ingress, and license review for any reused Pack patterns.
- **Effort:** **M/L**.
- **Compatibility/security constraints:** declare the Parler room log authoritative for Parler
  message/cursor state and the extmsg transcript authoritative for Gas session membership/read
  state; do not mirror either by last-write-wins. Keep credentials separate, verify Parler
  signature/target/membership/sender policy before normalized ingress, protect the stable
  controller listener or configure current-main admission grants, preserve provider-ID dedupe, use
  loop markers and a durable outbox, and record that bridge-signed output is authored by the
  bridge. Parler's native visible adapter must not compete for a Gas-owned session. No work closes
  on delivery alone.
- **Success metric:** 100 deliveries with one repeated source event ID create one admitted extmsg
  transcript record and one bridge transition; a lost reminder for a successfully appended routed
  entry remains recoverable through transcript backfill. Forced hydration-pending input does not
  become executable without a durable entry. A controller restart re-registers the adapter and
  reconciles durable bindings/outbox state; an in-flight unregister failure is visible. Crash tests
  retry before admission, release an explicit pre-action rejection, and make
  post-persisted-replay-admission recovery manual and visible without double execution. Ambiguous
  post-send correlation is surfaced, not called exactly-once. Operators can disable the adapter
  without migrating Beads, Gas City, or Parler data.
- **Stop/go criterion:** **Go** beyond experimental only if three active teams use it weekly for
  eight weeks, N3 remains green, and hydration-pending cannot bypass durable append. **Stop** if
  most support incidents require Parler to understand formulas, controller state, or provider
  login, or if the extmsg seam cannot enforce persist-before-wake.

#### X4. Expand visible-host parity from measured demand

- **Outcome:** add the single highest-demand visible host adapter and publish its exact wake,
  catch-up, attachment, result, and failure semantics.
- **Evidence:** visible normal turns are a real differentiator from database mail and runtime
  nudges, but only three hosts have native parity today.
- **Product layer:** CLI visible adapter plus shared `AdapterContext` contract tests.
- **Dependencies:** N1 support matrix, N3 harness, verified native injection/transcript APIs, and a
  demand/supportability score.
- **Effort:** **M/L** per host.
- **Compatibility/security constraints:** do not simulate parity with polling output; preserve the
  provider's permission channel and one activation consumer per identity/room; require signed
  backlog validation, bounded catch-up, target filtering, loop prevention, and terminal result
  publication. Commit actionable input only after native acceptance or intentional policy
  resolution.
- **Success metric:** the new adapter passes the full visible-host contract and survives reconnect,
  duplicate input, attachment failure, and host restart; at least 20 percent of measured unsupported
  visible-host demand selected it before implementation.
- **Stop/go criterion:** **Go** one host at a time when native APIs are stable. **Stop** if the only
  injection path edits user transcripts unsafely or cannot distinguish accepted context from
  display output.

### Later: 6–12 months, conditional bets

#### L1. Research cross-hub authenticated relay and history semantics

- **Outcome:** a threat model, consistency model, protocol sketch, and simulator answer whether
  agents on separate authoritative hubs can exchange signed messages without false global-order or
  delivery claims.
- **Evidence:** Beads/Wasteland show demand for sovereign state and also expose merge, duplicate
  claim, credential, and migration costs. Parler currently has no cross-hub implementation.
- **Product layer:** research/specification first; no production commitment.
- **Dependencies:** at least three real multi-hub deployments blocked by the current model; X2
  transcript-verification conclusions; key discovery and revocation model.
- **Effort:** **L** research, larger implementation.
- **Compatibility/security constraints:** preserve self-certifying identity, explicit hub
  membership, durable receiver cursor truth, plaintext-hub honesty, and partition/split-view
  disclosure. Do not reuse database merge as message delivery.
- **Success metric:** a simulator and conformance model specify behavior under partition,
  duplication, reordering, malicious relay, retention mismatch, and hub loss; external reviewers
  can state the guarantees without disagreement.
- **Stop/go criterion:** **Go** to implementation only if measured deployments need it and the model
  stays additive. **Stop** if it requires a global trust root, pretends to offer one total order, or
  makes the one-hub beginner path harder.

#### L2. Consider a native Gas City runtime/provider integration

- **Outcome:** only if the external bridge proves durable demand, evaluate whether a Parler-aware
  provider can improve session status/wake/result reporting beyond the extmsg transport adapter.
- **Evidence:** Gas City exposes runtime providers, but its process authority and Parler's
  trust-aware conversation authority are different. A deeper bridge might remove glue only after
  the boundary is stable.
- **Product layer:** Gas City provider plugin maintained with that ecosystem; minimal Parler SDK
  additions only if generally useful.
- **Dependencies:** X3 go criterion, stable provider API across two Gas City releases, joint
  maintenance ownership.
- **Effort:** **L**.
- **Compatibility/security constraints:** Gas City owns desired/running state; Parler owns its
  signed-message, room, and cursor semantics. Provider credentials never contain the agent seed.
  Restart follows the N3 boundaries: pre-admission work may retry, while post-persisted-admission
  ambiguity fails closed with visible recovery rather than silent reinjection.
- **Success metric:** native integration removes at least 50 percent of bridge-specific
  configuration/support incidents while preserving all N3 failure guarantees.
- **Stop/go criterion:** **Go** only with an upstream-compatible extension seam and a named
  maintainer on each side. **Stop** if the external transport adapter remains simpler or release
  cadence causes repeated breakage.

#### L3. Evaluate receipt-derived evidence, offers, and reputation

- **Outcome:** determine whether signed request/result pairs can support useful capability evidence
  without turning self-report, collusion, or hub visibility into a misleading score.
- **Evidence:** Parler ships signed terminal receipts but not trustworthy aggregation; Wasteland's
  Phase 1 trust limits show the danger of presenting record existence as enforced reputation.
- **Product layer:** analytics/privacy/security research; optional directory projection only after
  validation.
- **Dependencies:** enough real receipts, explicit consent/retention rules, task-authority
  references, anti-collusion analysis, and an appeals/correction model.
- **Effort:** **L**.
- **Compatibility/security constraints:** never rank agents from self-reported token/time fields
  alone; do not expose private room/task content; distinguish signed claim, requester acceptance,
  and independently verified outcome.
- **Success metric:** a blinded evaluation shows the derived signal predicts externally verified
  outcomes better than self-description, with documented false-positive and gaming rates.
- **Stop/go criterion:** **Go** only if evidence beats a simple verified-history count and privacy
  review passes. **Stop** if the metric can be cheaply inflated by two cooperating identities or
  cannot explain corrections.

### Roadmap sequence

```text
N1 capability matrix ──────────────┐
                                   ├─> X1 supported adapter kit
N2 Beads reference prototype ──────┤             |
                                   |              +─> X3 Gas City bridge ─> L2 native provider?
N3 conformance harness ────────────┘

N4 compact context/tool surface
N5 blob verification

measured host demand ─────────────────────────────> X4 next visible host

X2 transcript verification ────────────────────────────────> L1 federation research?

real signed-receipt volume ─────────────────────────────────> L3 evidence/reputation research?
```

The arrows are gates, not promises. In particular, federation, native runtime integration, and
reputation should disappear from the roadmap if their stop criteria are met.

## Adversarial checks and disputed claims

| Tempting claim | Evidence against a simple version | Report position |
|---|---|---|
| “Beads is only a task tracker” | Broad issue model, memory/prime, message type, formulas, wisps, gates, tracker sync | Call it a work/memory ledger, then distinguish transport/runtime |
| “Beads provides agent messaging” | `bd mail` delegates; no core presence, durable receiver cursor, visible turn, or cryptographic author path found | It stores message-shaped state; delivery belongs elsewhere |
| “Gas City replaced/deprecated Gas Town” | Gas City says it extracted the machinery and Gas Town is among orchestrators built on it | Call Gas City the platform/successor path, not a formal replacement |
| “Gas City nudge delivers work” | Primary docs call it fire-and-forget and lost when the session is absent | Durable work and default-provider mail recover; nudge only reduces latency |
| “Wasteland has distributed claim locks” | Phase 1 wild-west permits duplicate local claims; current PR mode isolates branch proposals rather than globally locking | Qualify by mode/backend; do not claim a distributed lock |
| “Dolt makes coordination conflict-free” | Same-cell lab edit required operator resolution; release history required migration/schema repair | Dolt offers useful cell merge/history with explicit conflict/upgrade costs |
| “Known Beads defects disprove ledgers” | The defects concern migrations, mode parity, init, and concurrency implementation; current releases add repairs | Treat them as design-history evidence, not category disproof |
| “Stars prove production adoption” | No public DAU, retention, deployment, or unique-user data; channels overlap | Report each dated signal separately and leave production use unverified |
| “Parler is private/E2E” | Hub sees plaintext by design | Say authenticated, scoped, and private-by-membership, never E2E from operator |
| “Parler wakes any AI host” | Only three visible adapters ship; MCP wiring alone cannot inject a visible model turn | Name supported hosts and publish the two support matrices together |
| “Parler has exactly-once agent execution” | Idempotent send and replay guards do not prove a model action or external side effect | Claim durable delivery and duplicate suppression at their exact boundaries |
| “Portable keys or A2A cards are federation” | They route to one hub/project a discovery card; no history replication exists | Keep federation unbuilt until a consistency model and demand exist |

DoltHub's
[multi-agent persistence essay](https://www.dolthub.com/blog/2026-03-13-multi-agent-persistence/)
corroborates the work-ledger/orchestration category split, but it is vendor-adjacent advocacy for
Dolt and is not treated as neutral evidence that Dolt is the best storage for Parler.

## Open questions

1. How many Parler users already use Beads, Gas Town, Gas City, or another tracker? Instrumentation
   should be opt-in and privacy-preserving; maintainer interviews may answer faster.
2. Does an opaque external task reference plus readable text cover real workflows, or do users need
   tracker snapshot fields in a namespaced extension?
3. Which external transition, if any, should a signed `done` receipt request? Automatic close may
   be wrong when review/gates remain; comment-only may be the safe default.
4. How stable are Gas City's extmsg transport-adapter, Pack, event-provider, and runtime-provider
   APIs across two releases?
5. Who would maintain an independent Gas City adapter and optional Pack, and what license should
   the Pack use given the upstream Packs repository's missing root license at the snapshot?
6. What is Parler's actual advanced MCP tool usage and schema-byte distribution by host?
7. Which unsupported host has both user demand and a safe native visible-injection API?
8. Which independent checkpoint-comparison channel can expose equivocation without introducing a
   central trust root or breaking retention?
9. Do any real deployments need agents on different hubs to share one conversation, or is
   `KEY@HUB` operationally sufficient?
10. Can receipt-derived evidence distinguish a worker's signed claim from requester acceptance and
    independent verification without leaking private work?

## Source register

All web sources were retrieved on 2026-07-21 MDT / 2026-07-22 UTC. Immutable commit links are used
for architectural claims wherever possible.

### Beads primary sources

- [Stable `v1.1.0` release and migration instructions](https://github.com/gastownhall/beads/releases/tag/v1.1.0)
- [Stable source tree](https://github.com/gastownhall/beads/tree/8e4e59d39f3459a43cf21a3236a13eca4dd874f7)
- [Current-main project charter](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/engdocs/PROJECT_CHARTER.md)
- [Current issue model](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/types/types.go)
- [Stable create command](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/cmd/bd/create.go)
- [Stable shared create operation](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/storage/issueops/create.go)
- [Stable migration-hygiene gate](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/scripts/check-migration-hygiene.sh)
- [Stable federation command](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/cmd/bd/federation.go)
- [Current guarded issue operations](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/dolt/issues.go)
- [Current leases and tests](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/dolt/lease_test.go)
- [Current federation credential protection](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/internal/storage/dolt/credentials.go)
- [Current messaging boundary](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/engdocs/messaging.md)
- [Stable MCP server](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/integrations/beads-mcp/src/beads_mcp/server.py)
- [Stable metrics implementation](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/internal/metrics/metrics.go)
- [Security guidance](https://github.com/gastownhall/beads/security)
- [Contribution policy](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/CONTRIBUTING.md)
- [Maintainer review/audit policy](https://github.com/gastownhall/beads/blob/607df586d198400e2a65681b824b7d6e21a14d97/PR_MAINTAINER_GUIDELINES.md)
- [Stable curated community tools](https://github.com/gastownhall/beads/blob/8e4e59d39f3459a43cf21a3236a13eca4dd874f7/docs/COMMUNITY_TOOLS.md)

### Gas Town, Gas City, Packs, and Wasteland primary sources

- [Gas Town architecture](https://github.com/gastownhall/gastown/blob/67a8d72a7aa415cad5b9832bdbba31b6ec026417/docs/design/architecture.md)
- [Gas City transition guide](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/docs/getting-started/coming-from-gastown.md)
- [Gas City dispatch architecture](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/dispatch.md)
- [Gas City sling implementation](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/cmd/gc/cmd_sling.go)
- [Gas City mail/nudge semantics](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/messaging.md)
- [Gas City event bus](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/event-bus.md)
- [Stable Gas City external-messaging design](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/engdocs/design/external-messaging-fabric.md)
- [Stable Gas City shared-thread design](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/engdocs/design/external-messaging-shared-threads.md)
- [Stable Gas City external-messaging types](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/internal/extmsg/types.go)
- [Stable Gas City external-messaging handlers](https://github.com/gastownhall/gascity/blob/8ffc009ded781a2ada2077f3a29bd712b2def0bf/internal/api/handler_extmsg.go)
- [Current connected-client guide](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/docs/guides/connected-clients.md)
- [Connected-client spec-first release gate](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/release-gates/ga-zy0p7n-connected-client-docs-gate.md)
- [Gas City current-main write grants](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/internal/api/writeauth.go)
- [Gas City current-main read grants](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/internal/api/readauth.go)
- [Gas City sessions/providers](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/session.md)
- [Gas City health/supervision](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/engdocs/architecture/health-patrol.md)
- [Gas City Pack specification](https://github.com/gastownhall/gascity/blob/2abd12e857a2c38875db51b681736a4e053b89b1/docs/reference/specs/pack-spec.md)
- [Pinned Gas City Packs tree](https://github.com/gastownhall/gascity-packs/tree/56d07c537b21d9db239ed6f9e17e6c6e37da0864)
- [Gas Town Wasteland Phase 1 document](https://github.com/gastownhall/gastown/blob/67a8d72a7aa415cad5b9832bdbba31b6ec026417/docs/WASTELAND.md)
- [Wasteland current-main tree](https://github.com/gastownhall/wasteland/tree/fbad824c1e826be00ec7e8434473e6a70556a6fd)
- [Wasteland mode-aware mutation path](https://github.com/gastownhall/wasteland/blob/fbad824c1e826be00ec7e8434473e6a70556a6fd/internal/sdk/mutate.go)
- [Stable Wasteland signed commit path](https://github.com/gastownhall/wasteland/blob/5535bcdc3a368df0d4c9025dcdccc237ccf49e6e/internal/commons/commons.go)
- [Stable Wasteland verification command](https://github.com/gastownhall/wasteland/blob/5535bcdc3a368df0d4c9025dcdccc237ccf49e6e/cmd/wl/cmd_verify.go)
- [Wasteland GPG verification command](https://github.com/gastownhall/wasteland/blob/fbad824c1e826be00ec7e8434473e6a70556a6fd/cmd/wl/cmd_verify.go)

### Parler implementation and tests

- [Identity and seed handling](../../crates/parler-auth/src/identity.rs)
- [Wire frames, signatures, handoffs, tasks, and claims](../../crates/parler-protocol/src/hub.rs)
- [Additive extension codec](../../crates/parler-protocol/src/types.rs)
- [Connector send, verify, reconnect, receive, and acknowledgement](../../crates/parler-connector/src/agent.rs)
- [Attention, replay guard, and host-wake boundary](../../crates/parler-connector/src/runtime.rs)
- [Real hub/connector E2E workflows](../../crates/parler-connector/tests/mesh_e2e.rs)
- [Hub authentication/routing](../../crates/parler-hub/src/server.rs)
- [Hub SQLite persistence, claims, memory, and blobs](../../crates/parler-hub/src/store.rs)
- [Visible conversation adapter core](../../crates/parler-cli/src/conversation.rs)
- [Managed worker boundary](../../crates/parler-cli/src/worker.rs)
- [Task lifecycle contract](../task-lifecycle.md)
- [Visible host support contract](../visible-host-adapters.md)
- [Storage/memory status](../storage-and-memory.md)
- [Non-authoritative project backlog](../../tasks/backlog.md)

### Adoption and independent-origin sources

- [Beads Homebrew analytics](https://formulae.brew.sh/formula/beads)
- [Gas Town Homebrew analytics](https://formulae.brew.sh/formula/gastown)
- [Beads GitHub repository snapshot](https://github.com/gastownhall/beads)
- [Gas Town GitHub repository snapshot](https://github.com/gastownhall/gastown)
- [Gas City GitHub repository snapshot](https://github.com/gastownhall/gascity)
- [PyPI `beads-mcp`](https://pypi.org/project/beads-mcp/)
- [Hacker News Gas Town discussion](https://news.ycombinator.com/item?id=46458936)
- [DoltHub multi-agent persistence essay](https://www.dolthub.com/blog/2026-03-13-multi-agent-persistence/)

## Final recommendation

Parler can win a durable category by being narrow:

- let Beads and human trackers own work truth;
- let Gas City and other orchestrators own runtime truth;
- let Wasteland or future markets own cross-organization work/reputation truth;
- make Parler the best place for verifiable public-key authorship, durable receiver state, and a
  live shared conversation across independent agent hosts.

Start with a capability matrix, an external Beads reference prototype, generic blob verification,
and a failure-injection conformance harness. Those moves are useful even if the integration is never
productized. Build a Gas City bridge only after repeated usage, and treat federation and reputation
as conditional research rather than destiny.
