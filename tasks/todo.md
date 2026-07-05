# Brand rename: "Parler" → "Parler Protocol" (surface only)

Goal: rename the human-facing **brand name** everywhere a person reads it (docs, website,
images, CLI/hub output, package metadata). Defer **code identifiers** per the advisor note.

## Rules

Replace the standalone capitalized word `\bParler\b` → `Parler Protocol`. This automatically:
- MATCHES brand prose, "Parler hub", "Parler-aware" (fine).
- SKIPS lowercase `parler`, `parler-*` crates, `parler_*` tools, `PARLER_*` env vars, `com.parler.*`
  (all lowercase → a capital-P word match never hits them).
- SKIPS compound identifiers `ParlerApi`, `appParlerHome`, `ParlerHub` (no word boundary after
  "Parler" → no match).
- No doubling: repo currently has ZERO existing "Parler Protocol"/"Parler protocol" strings.

## Do NOT touch (deferred code identifiers)
- Crate names / dirs `parler-hub|cli|connector|protocol|auth|bin`
- MCP tool names `parler_*`; Env vars `PARLER_*`; Reverse-DNS `com.parler.*`
- Binary invocation `parler`, and the `#[command(name="parler-hub")]` value
- Domain `parlerprotocol.com` (already correct)
- Asset filenames (e.g. `docs/assets/parler-banner.svg` — lowercase, not shown to users)

## Steps
- [ ] 1. Perl word-boundary replace across curated text-file set (md/tsx/ts/rs/toml/mmd/yml/html)
- [ ] 2. Hand-edit the 6 SVGs (logo/banner/blog-card text) + check text width
- [ ] 3. Verify the `server.rs` HTML-assert pair moved together; no other coupled asserts
- [ ] 4. Tighten the 2-3 most prominent redundant taglines
- [ ] 5. Review full `git diff` for over-replacement (identifiers, URLs, slugs)
- [ ] 6. `CI_SKIP_WEB=1 make ci` (rust) + web build green

## Review

Renamed the human-facing brand **Parler → Parler Protocol** across 114 files (+450/-453),
leaving every code identifier intact. Method: case-sensitive word-boundary replace
`\bParler\b → Parler Protocol` with a `(?!\.(app|dmg))` guard, over a curated list built from
`git grep -lw Parler`. Because the match is capital-P and whole-word, it never touched
`parler` / `parler-*` crates / `parler_*` tools / `PARLER_*` env vars / `com.parler.*` /
compound `ParlerApi` (no word boundary). Post-sweep grep for leftover bare-brand `Parler`:
**zero** — every remainder is a deliberately deferred artifact/identifier.

Hand-handled edge cases:
- `packaging/homebrew/parler.rb` — kept `class Parler` (Ruby class bound to filename); renamed
  only the header comment.
- `desktop/electron-builder.yml` `productName: Parler` + all `Parler.app`/`Parler.dmg` literals
  — DEFERRED (app-bundle artifact name, same bucket as the binary/Fly app).
- `docs/assets/parler-banner.svg` — redesigned the wordmark lockup (font 56→46, recentred) so
  "Parler Protocol" fits; verified by rasterizing with qlmanage.
- 3 blog-card SVGs — footer captions only, layout-safe.
- `server.rs` HTML string + its `assert!` moved together; hub instance names
  (`PARLER_HUB_NAME` defaults + fly/compose) + their coupled test fixtures renamed in lockstep.

Verified: web `npm run build` green (all routes prerendered); Rust `CI_SKIP_WEB=1 make ci`
green (see below).

DEFERRED per advisor note (code identifiers, separate post-sprint job): crate names, `parler`
binary, `parler_*` MCP tool names, `PARLER_*` env vars, `com.parler.*` wire ids, Fly app name,
desktop `.app`/`.dmg` artifact name, Homebrew Ruby class.

NOT changed (editorial, flagged to user): the SEO keyphrase "chat protocol for AI agents" in
blog deks now reads "Parler Protocol … chat protocol" (mild redundancy) — left intact to avoid
gutting search copy.
