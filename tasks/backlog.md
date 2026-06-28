# Backlog — the autonomous loop's work queue

This is the **forward queue**: a prioritized list of small, independently-shippable items the loop
pulls from, one per iteration (top unchecked item first). It is the single source of truth for "what
to work on next".

- `tasks/todo.md` is the **log** of finished work — append a summary there when you complete an item.
- `tasks/lessons.md` is the **memory** — append a rule there after any correction or surprise.

The **`web/` app is out of scope** for this loop — Tam drives it by hand. Loop items must be
Rust/CLI/protocol only; anything that also needs a UI/site change carries a `[HUMAN] web: …` note for
the part the loop leaves untouched. The loop gates with `scripts/verify.sh --rust-only`.

Each item must be small enough to land behind `scripts/verify.sh --rust-only` in one iteration, additive,
and backward-compatible with the deployed protocol/hub. If an item is too big, the loop should split it
and check in the sub-items here rather than attempt it whole. Keep `[P0]`/`[P1]`/`[P2]` priority tags.

> **Editing rules:** add new items at the right priority; never delete history (check items off with
> `[x]` and let `todo.md` carry the write-up). Anything referencing the pre-pivot NATS architecture
> (`parler-manager`, `parler-delivery`, `parler-console`, KV planes…) is **dead** — those crates were
> removed in cc686ea. Do not resurrect them.

---

## Now (pull from the top)

- [ ] **[P0] Seed `tasks/lessons.md` discipline** — on the *first* real code iteration, confirm the
  verify gate runs clean on a no-op, then proceed. (Sanity check that the loop's feedback signal is
  trustworthy before trusting it to gate commits.)

- [ ] **[P1] Code-handoff frontier index** (`docs/code-handoff.md` Phase 3) — index the latest bundle
  per room (tip id / short summary / author / ts) in the hub store; expose `parler frontier --room R`
  on the CLI; surface "latest handoff" in `parler rooms` output. *Done when:* new store table/columns
  + migration, CLI subcommand, an e2e test that pushes two bundles and asserts `frontier` returns the
  second, and the README/`docs/code-handoff.md` Phase 3 box is checked. Additive only.

- [ ] **[P1] Streaming blob upload** (`docs/storage-and-memory.md` P3 / B1) — replace the single
  fully-buffered-in-RAM blob frame with chunked upload so large handoffs don't pin memory. Keep the
  25 MiB cap as a configurable ceiling; verify sha256 incrementally. *Done when:* protocol frames for
  chunked put, hub assembles to disk without buffering the whole blob in RAM, connector streams from a
  file, and an e2e test moves a >1 MiB bundle in chunks. Backward-compatible: old single-frame path
  still accepted.

## Next

- [ ] **[P2] sqlite-vec semantic memory** (`docs/storage-and-memory.md` P4) — this needs a client
  embedding source that does not exist yet, so it is **blocked**: land it only as a self-contained
  follow-up so the deployed protocol isn't left half-changed. Until unblocked, leave checked-off-able
  design notes only. *Prereq:* decide where embeddings come from (client-supplied vs hub-side model).

- [ ] **[P2] schemars schema export** — `parler-protocol`: generate `spec/parler.schema.json` from the
  frame types via `schemars`, plus a test that the checked-in schema matches the generated one (so the
  wire format can't drift silently). *Done when:* schema file + drift test in CI's `cargo test`.

## Icebox (needs a human decision before the loop touches it)

- [ ] Benchmarks vs the old Node implementation (criterion + e2e RTT/throughput) → `docs/benchmarks.md`.
- [ ] Anything that changes the deployed wire protocol in a non-additive way (needs explicit sign-off).
