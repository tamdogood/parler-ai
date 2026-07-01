# Task: Standalone full-screen Agents Console page (web) — 2026-06-29

**User ask:** from the website, build an *extra standalone page* for the agents hub; on that page add
*more agent-focused features* and make the *existing agents features (the directory) occupy most of
the screen*.

## Design — **Option A** (user-chosen): one `/hub` page, two tabs (Agents + Sessions)
Build on the existing REST surface only (`/api/hub`, `/api/directory`, `/api/session`). Reuse
`AgentCard`, `AgentDetail`, `TokenDialog`, `StatusDot`, design tokens. Agents tab uses a faceted-
search model: fetch the scope+query set once, then filter **status + tags client-side** so all the
live counts stay coherent. Sessions tab = "session hub" = the sessions explainer + the watch viewer.

New:
- [x] `components/agents-console.tsx` — full-width (`max-w-[1600px]`) console: sticky left filter rail
      (scope · status facets w/ counts · tag facets w/ counts · token) + dominant main column.
      New features vs. home Directory: headline live metrics (agents · online · public · verified),
      **sort** (recent/name/status), **grid⇄list toggle**, **"Live activity"** strip, up-to-4-col grid.
- [x] `components/sessions-feature.tsx` — extracted from home `Sessions()` (`showViewerCta` prop).
- [x] `components/session-viewer.tsx` — extracted watch viewer from `app/session/page.tsx`.
- [x] `components/session-hub.tsx` — Sessions tab = `<SessionsFeature/>` + `<SessionViewer/>`.
- [x] `app/hub/page.tsx` — standalone tabbed page (hash-synced: `/hub` agents, `/hub#sessions`).

Modify:
- [x] `app/page.tsx` — use `<SessionsFeature/>`; prune now-unused imports.
- [x] `app/session/page.tsx` — client redirect → `/hub#sessions` (carry any `&k=` watch token).
- [x] `nav-bar.tsx` — add "Hub" link + repoint CTA + session-viewer link to `/hub`.
- [x] `directory.tsx` + `hero.tsx` (home) — link out to `/hub`.
- [x] Verify: `cd web && npm run build && npm run lint` green; grep no stale `/session` links.

## Review — DONE (2026-06-29) ✅ `next build` green (9 routes prerender, /hub 13.2 kB)
Shipped **Option A**: a standalone `/hub` page with **Agents** + **Sessions** tabs, additive (home
page and REST surface untouched — no hub/protocol change).
- **Agents tab** (`components/agents-console.tsx`): full-width `max-w-[1600px]` console so the directory
  dominates the viewport. Sticky left rail (scope · status facets w/ live counts · tag facets w/ counts
  · token) + a main column with: headline metrics (agents · online · public · verified), a **Live
  activity** strip (working/waiting agents + their `activity`), **search**, **sort** (recent/name/
  status), **grid⇄list toggle**, up-to-4-col grid, and a scannable list view. Faceted-search model:
  fetch the scope+query set once, facet status/tags client-side so every count stays coherent.
- **Sessions tab = "session hub"** (`components/session-hub.tsx`): the sessions explainer
  (`sessions-feature.tsx`, extracted from the home `Sessions()`) + the watch viewer
  (`session-viewer.tsx`, extracted from the old `/session` page) on one screen — exactly the requested
  "combine Session viewer with session."
- **Routing/wiring:** `app/hub/page.tsx` (hash-synced tabs: `/hub`, `/hub#sessions`, deep-link
  `/hub#sessions&k=<token>` opens the viewer pre-connected). Old `/session` → client redirect carrying
  the watch token. NavBar gains "Hub" + repoints the CTA; home Directory + Hero link out to `/hub`.
  Viewer hash writes use `replaceState` so tab switches never scroll-jump to the `#sessions` anchor.
- **Verified:** `npm ci && npm run build` clean (type-check passes, no orphan imports); `next start`
  smoke — `/hub` 200 (both tabs render), `/session` 200 (redirect copy), `/` 200; grep shows no stale
  `/session` links.

---

# Task: SEO — make the Parler website discoverable — 2026-06-29

**User ask:** "how to improve SEO for my website to make it more discoverable?" → plan + implement.

## Findings (current state of `web/`)
- Next 15 App Router. Root `layout.tsx` sets only `title` + `description` + `metadataBase`
  (`https://parler-hub.fly.dev`). No OG, no Twitter card, no og:image.
- Blog `[slug]` has `generateMetadata` with `openGraph` but no Twitter card, no canonical, no
  article metadata, no JSON-LD.
- No `sitemap.xml`, no `robots.txt`.
- No structured data anywhere (we have a full FAQ component + an Article — both free rich-result
  wins).
- `/session` is a dynamic, thin, auth-gated viewer page that is currently indexable.

## Plan
- [ ] `web/lib/seo.ts` — single source of truth: `SITE_URL`, site name/description, and the
      `WebSite` + `SoftwareApplication` JSON-LD objects.
- [ ] `web/app/robots.ts` — allow all, declare sitemap, disallow `/session`.
- [ ] `web/app/sitemap.ts` — `/`, `/blog`, and every post from `POSTS` (lastModified = post.date).
- [ ] `web/app/opengraph-image.tsx` — dynamic on-brand 1200×630 OG image (next/og, default font).
- [ ] `web/app/twitter-image.tsx` — re-export the OG image so Twitter gets a card image too.
- [ ] `web/app/layout.tsx` — expand root metadata (openGraph, twitter `summary_large_image`,
      canonical, keywords, authors/creator) + inject WebSite/SoftwareApplication JSON-LD.
- [ ] `web/app/blog/[slug]/page.tsx` — add Twitter card, canonical, article publishedTime/authors;
      inject `BlogPosting` JSON-LD.
- [ ] `web/app/blog/page.tsx` — add openGraph + canonical to the index.
- [ ] `web/components/faq.tsx` — add plain-text answers + emit `FAQPage` JSON-LD.
- [ ] `web/app/session/layout.tsx` — server layout exporting `robots: { index: false }` (page is a
      client component, so it can't export metadata itself).

## Verify
- [ ] `npm run build` in `web/` is green (renders the dynamic OG image, validates metadata).
- [ ] Spot-check generated routes for `/sitemap.xml`, `/robots.txt`, og image.

## Review
Done. `npm run build` green; new routes `/sitemap.xml`, `/robots.txt`, `/opengraph-image`,
`/twitter-image` all prerender. Verified in the built HTML:
- Homepage: canonical + full OG + `twitter:summary_large_image` + auto-injected OG/Twitter image;
  JSON-LD `WebSite` + `SoftwareApplication` + `FAQPage` (Q/A) present.
- Blog post: canonical, Twitter card, `BlogPosting` JSON-LD.
- `robots.txt`: allow `/`, disallow `/session`, sitemap + host declared.
- `/session`: `<meta name="robots" content="noindex, nofollow">`.

New files: `lib/seo.ts`, `app/robots.ts`, `app/sitemap.ts`, `app/opengraph-image.tsx`,
`app/twitter-image.tsx`, `app/session/layout.tsx`.
Edited: `app/layout.tsx`, `app/blog/page.tsx`, `app/blog/[slug]/page.tsx`, `components/faq.tsx`.

Not done (off-page / content — out of code scope): submit sitemap to Google Search Console + Bing,
write more blog posts, earn inbound links, move to a real domain (vs `*.fly.dev`).

---

# Task: Seamless private hub — "docker compose up, agents talk in no time" — 2026-06-29

**User ask:** make the **private** (self-hosted) hub as easy to stand up as the public hub. "As easy
as running a docker to the database, and the agents can just talk to each other in no time." Goal is
adoption — setup must be one command on the operator side and a copy-paste snippet on the agent side,
**symmetric with the public hub** and **without weakening the security model**.

## Today's asymmetry (the gap)
- Public hub onboarding = `claude mcp add parler -- parler mcp` (URL baked in; MCP self-bootstraps).
- Private hub: `deploy/` is titled "Deploy the **public** hub"; both recipes (Fly, VPS+Caddy) assume
  public + a domain + TLS. "Private" is a one-line footnote ("drop `--public`"). There is **no**
  one-command private recipe, and the runtime image is **distroless (no shell)** so a wrapper script
  can't generate a secret. A LAN-reachable private hub *should* set a join secret (security invariant),
  but inventing + distributing one by hand is friction.

## North-star experience (symmetric, one command each side)
```
# Operator, one box:
docker compose -f deploy/private/docker-compose.yml up -d
#   → boot log prints the exact connect line, with the auto-generated secret:
#     PARLER_HUB=ws://localhost:7070 PARLER_JOIN_SECRET=<gen> claude mcp add parler -- parler mcp
# Each agent:
PARLER_HUB=ws://<host>:7070 PARLER_JOIN_SECRET=<gen> claude mcp add parler -- parler mcp
```

## Design decisions
- **Auto-generated, persistent join secret via a file** (the key enabler). New flag
  `--join-secret-file` / env `PARLER_HUB_JOIN_SECRET_FILE`: read the secret from the file; if absent,
  generate a strong one (reuse the hub's existing token generator), persist it `0600`, reuse on later
  boots. Precedence: explicit `--join-secret` value > file > none. **Binary default is unchanged**
  (no secret unless asked) — this is opt-in and only the private compose sets it. Solves seamless +
  secure-by-default + distroless (no shell needed) in one small, testable helper.
- **Mode-aware landing page + boot banner.** The boot banner (stdout = operator-only) prints the
  ready-to-paste connect line *with the real secret*. The `GET /` page is world-reachable, so it must
  **never print the secret** — for a private hub it shows the snippet with a `PARLER_JOIN_SECRET=<your-
  join-secret>` placeholder and points the operator at the boot log / secret file. Map a `0.0.0.0`
  bind → `localhost` for display so the snippet is copy-pasteable on the common same-machine case.
- **`deploy/private/`** — hub-only compose (no Caddy/domain/TLS), private mode, `7070:7070`, named
  volume, `PARLER_HUB_JOIN_SECRET_FILE=/data/join-secret`. Reuses `deploy/Dockerfile`.
- **Out of scope:** `web/` (private directory viewing already works via tokens); a prebuilt GHCR image
  (truest `docker run`, but touches release/CD + registry namespace — offer as a follow-up).

## Steps
- [x] Hub lib: `secret::resolve_join_secret` + `random_secret` (generate-if-absent, persist `0600`,
      reuse). 6 unit tests.
- [x] `main.rs`: `--join-secret-file` arg; precedence (explicit > file > none); private connect banner.
- [x] `server.rs`: `landing_html(requires_secret)` — private copy + `PARLER_JOIN_SECRET=<placeholder>`
      (structurally can't leak the real secret); `0.0.0.0`/`[::]`→`localhost` in `display_hub_url`. Tests.
- [x] `deploy/private/docker-compose.yml` (hub-only, `command: []` ⇒ private, secret-file) + README.
- [x] Docs: README "Option C"; reframed `deploy/README.md`; AGENTS pointer row.
- [x] `CI_SKIP_WEB=1 make ci` green; booted the real binary twice (generate→persist `0600`→reuse +
      banner with the live secret); compose resolves to `command: []`; public compose still `--public`.

## Review
**Done & verified.** Private-hub onboarding is now symmetric with the public hub: one command on the
box, one copy-paste line per agent — and the hub hands you that exact line.

- **Operator:** `docker compose -f deploy/private/docker-compose.yml up -d --build`. Boots PRIVATE,
  auto-generates + persists a join secret (`/data/join-secret`, `0600`, stable across restarts), and
  prints `PARLER_HUB=… PARLER_JOIN_SECRET=… claude mcp add parler -- parler mcp` in its log.
- **Agent:** paste that line. (`parler mcp` already self-bootstraps; client already reads
  `PARLER_JOIN_SECRET`.) Nothing else.
- **Security held / strengthened:** the world-reachable `GET /` never receives the secret (no param —
  shows a placeholder + "find it in the boot log"); the real secret only hits operator stdout/logs +
  the `0600` file. Private hubs now require a secret by default (was an open "drop --public" footnote).
- **Minimal blast radius:** binary default unchanged (feature is opt-in via `--join-secret-file`); no
  new runtime deps (tempfile is dev-only, already in-workspace); reused the shared Dockerfile + landing
  template. `parler-protocol` untouched, so no cross-crate ripple.

**Verification:** `CI_SKIP_WEB=1 make ci` → "all gates passed"; live binary proof (boot1 generated
`Pd9TW…RTgV`, persisted `0600`; boot2 reused the identical secret); `docker compose config` confirms
private=`command:[]`, public=`command:[--public]`.

**Follow-up SHIPPED — prebuilt GHCR image (`docker run …` in seconds, no compile):**
- `.github/workflows/release-image.yml` — multi-arch (amd64+arm64) build→push to
  `ghcr.io/<owner>/parler-hub` on a `v*` tag or manual dispatch. **No secrets, fork-safe** (pushes to
  the runner's own lowercased namespace via `GITHUB_TOKEN` + `packages: write`); tags via
  `docker/metadata-action` (`latest` / semver / `MAJOR.MINOR` / short-SHA). actionlint + selftest clean.
- **Made the image private-by-default** (the right posture for a published image — a bare `docker run`
  must not open a world-joinable hub). `deploy/Dockerfile` `CMD ["--public"]`→`CMD []`; default name
  →"Parler Hub". Kept the live Fly hub public **safely** via the *existing* `PARLER_HUB_PUBLIC` env
  (added `PARLER_HUB_PUBLIC = "true"` to `fly.toml` — verified `=true`→public, bare→private, `--public`
  arg→public on the real binary). Public compose unaffected (already passes `--public` explicitly).
- `deploy/private/docker-compose.yml` now `image: ghcr.io/tamdogood/parler-hub:latest` + `build:`
  fallback (`--build` from a clone). README/deploy/private + docs/ci-cd.md document the `docker run`
  path. Both composes verified via `docker compose config` (private=`command:[]`+secret, public=`--public`).
- Caveat: Docker daemon was down locally so the *image build* itself runs in CI; the Dockerfile delta
  is only the `CMD`/`ENV` lines (build otherwise identical to the proven Fly build) and the binary's
  mode selection is directly proven. `CI_SKIP_WEB=1 make ci` green.

---

# Task: SEO pass — apply the `astro-seo` skill's principles to the Next.js site — 2026-06-30

**User ask:** install `fusengine/agents astro-seo` via skillfish, then "apply this skill to improve SEO
for my website with your best effort." The skill is Astro-specific; the site is Next 15 App Router, so
we apply its *principles* (canonical correctness, RSS, sitemap, BreadcrumbList, feed autodiscovery,
XSS-safe JSON-LD). Existing SEO (PR #55/#56) is already strong (FAQPage/BlogPosting/OG+Twitter/sitemap/
robots), so this is a targeted improvement pass.

## Real bug found
- `/hub` (a `"use client"` page with no metadata) inherited the root layout's `alternates:{canonical:"/"}`
  → the standalone Hub self-reported as a duplicate of `/` and reused the home title/description.

## Plan
- [x] `lib/seo.ts` — `RSS_URL` + `ALT_RSS` feed-autodiscovery constant.
- [x] `app/layout.tsx` — drop root `canonical:"/"` (footgun: every un-overriding route inherited it);
      set site-wide `alternates.types` (RSS).
- [x] `app/page.tsx` — own `metadata` w/ `canonical:"/"` + RSS type.
- [x] `app/hub/layout.tsx` — NEW server layout: hub title/description/canonical `/hub`/OG/Twitter.
- [x] `app/sitemap.ts` — add `/hub`.
- [x] `app/blog/rss.xml/route.ts` — NEW static RSS 2.0 feed (XML-escaped, categories, atom:self).
- [x] `app/blog/page.tsx` — RSS alternate + `Blog` + `BreadcrumbList` JSON-LD.
- [x] `app/blog/[slug]/page.tsx` — RSS alternate + `BreadcrumbList` JSON-LD.
- [x] `components/footer.tsx` — RSS link.

## Verify
- [x] `npm run build` green (15 routes prerender; `/blog/rss.xml` + `/hub` both static).
- [x] Per-route canonicals correct: `/`→`/`, `/hub`→`/hub` (was `/` — the bug), `/blog`→`/blog`,
      post→own URL. `/hub` `<title>`/`og:title` now hub-specific, distinct from home.
- [x] `/blog/rss.xml` well-formed (`xmllint --noout` ✓): escaped titles/deks, categories,
      `atom:self`, RFC-822 dates. RSS `<link rel=alternate>` on home + blog pages; footer link.
- [x] `BreadcrumbList` JSON-LD on blog post + index; `Blog` collection JSON-LD on index.
- [x] Sitemap now lists `/hub`. `/session` still `noindex`; robots.txt unchanged.
- [x] Web CI gate = `scripts/ci/web.sh` (`npm ci` + `next build`); no `next lint` (no ESLint config).

## Review
**Done & verified.** Applied the `astro-seo` skill's *principles* to the Next.js site (skill is
Astro-only, so no Astro code — the checklist transferred: canonical correctness, RSS, sitemap,
BreadcrumbList, feed autodiscovery, XSS-safe JSON-LD via `dangerouslySetInnerHTML`+`JSON.stringify`).

- **Fixed a real canonical bug:** `/hub` (client page, no metadata) inherited the root layout's
  `canonical:"/"` and the home title/description — it self-reported as a duplicate of the homepage.
  Moved the home canonical off the root onto `app/page.tsx`, and gave `/hub` its own server
  `layout.tsx` (title/description/canonical/OG). Root now only advertises the feed site-wide, so no
  route inherits a wrong canonical.
- **Added an RSS 2.0 feed** (`/blog/rss.xml`, `force-static`) with autodiscovery `<link>`s + footer
  link. **Added BreadcrumbList** (posts + index) and a **Blog** collection schema. **Added `/hub`**
  to the sitemap.
- **Minimal blast radius:** `web/` only, no protocol/crate change; existing SEO (FAQPage, BlogPosting,
  OG/Twitter images, keywords) untouched.

New: `app/hub/layout.tsx`, `app/blog/rss.xml/route.ts`. Edited: `lib/seo.ts`, `app/layout.tsx`,
`app/page.tsx`, `app/sitemap.ts`, `app/blog/page.tsx`, `app/blog/[slug]/page.tsx`, `components/footer.tsx`.

Still off-page / out of code scope (same as the 2026-06-29 SEO task): submit sitemap to Google Search
Console + Bing, earn inbound links, a real domain vs `*.fly.dev`, more posts. Nice-to-have not done:
`Organization` logo node (no dedicated square-raster logo asset yet).

### Further pass ("anything else?") — DONE & verified
Recon showed the blog covers are a poor social-card source (aspect 1.14–2.36:1, none = OG's 1.91:1;
raw PNGs up to 3200px / ~400 KB via plain `<img>`), and there was no theme-color/manifest at all.
- **Per-post branded OG + Twitter cards** — `app/blog/[slug]/opengraph-image.tsx` (+ `twitter-image.tsx`
  re-export), 1200×630, title + dek on the root card's aesthetic, next/og default font. Both
  **prerender static** (`generateStaticParams`) so crawlers get a cached image. Dropped the manual
  `images:[post.cover]` from the post's `generateMetadata` so the branded card is the social image;
  the cover stays as the in-page hero + `BlogPosting` `image`. **Visually verified** the rendered PNG.
- **theme-color + web manifest** — `viewport` export (`themeColor:#000`, `colorScheme:dark`) →
  `<meta name=theme-color>`; `app/manifest.ts` → `/manifest.webmanifest` (Next auto-links it).
- **Image sitemap** — blog entries now carry `<image:loc>` (cover) for Google Images.
`next build` green (18 routes prerender). Verified in output: post `og:image`/`twitter:image` → the
branded `/blog/<slug>/opengraph-image` card; `theme-color` + `rel=manifest` present; manifest valid;
sitemap `<image:loc>` present.

Offered, not done (need a judgment call / visual QA): convert covers to `next/image` (Core Web Vitals —
they're 92–388 KB raw PNGs; touches rendering so wants visual QA); `Organization`/`publisher.logo`
(needs a light-bg square logo asset); AI-crawler policy in robots (a product decision).
