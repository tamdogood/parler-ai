# Task: Extensive Parler Protocol documentation on the website

The repo has rich docs in `docs/*.md` and a great README, but nothing user-facing on
the site explains *how to use* Parler. Add a proper `/docs` section to `web/`.

## Plan

- [ ] `web/lib/docs.ts` — page registry (slug, title, description, group, order) + prev/next helpers
- [ ] `web/components/docs/sidebar.tsx` — client sidebar, active-page highlight (usePathname)
- [ ] `web/app/docs/layout.tsx` — NavBar + sidebar + content + Footer shell
- [ ] `web/app/docs/page.tsx` — docs landing/overview (grouped cards)
- [ ] `web/app/docs/[slug]/page.tsx` — per-page header + body + prev/next, SEO + JSON-LD
- [ ] `web/components/docs/*.tsx` — content bodies:
  - introduction, quickstart, core-concepts
  - sessions, messaging, memory, file-and-code-handoff
  - reference (CLI + MCP tools + env vars)
  - self-hosting, security, troubleshooting
- [ ] Wire "Docs" into nav-bar + footer
- [ ] Add docs routes to sitemap.ts
- [ ] `cd web && npm run build` green

## Review — DONE

Added a full `/docs` section to `web/`, mirroring the blog infra (registry + BODIES map +
shared prose primitives) so it matches house style and stays maintainable.

- `lib/docs.ts` — 11-page registry, grouped, with prev/next helpers.
- `components/docs/sidebar.tsx` — client sidebar, active-page highlight (usePathname),
  collapses to a horizontal scroller on mobile.
- `app/docs/layout.tsx` — NavBar + sidebar + Footer shell (chrome can't drift between pages).
- `app/docs/page.tsx` — overview with grouped cards + ItemList/Breadcrumb JSON-LD.
- `app/docs/[slug]/page.tsx` — header + body + prev/next, per-page canonical/OG/TechArticle +
  Breadcrumb JSON-LD, generateStaticParams.
- 11 content bodies: introduction, quickstart, core-concepts, sessions, messaging, memory,
  file-and-code-handoff, reference (CLI + MCP + env), self-hosting, security, troubleshooting.
- Wired "Docs" into nav-bar + footer; added all 12 routes to sitemap.ts.

Content is drawn from README.md + docs/communication.md (authoritative), so commands/flags/env
vars/tool names are accurate.

Verified: `npm ci && npm run build` green — 65 pages, all 11 doc pages prerendered, no
type/lint errors. Sitemap emits all 12 /docs URLs; rendered HTML contains the expected copy;
nav Docs link present.

Not committed/pushed (no request to). Suggested follow-up: add a "Docs" CTA on the landing
hero, and OG images per doc page (blog has them; docs currently inherit the site default).
