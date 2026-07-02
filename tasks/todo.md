# Task: New SEO blog post — technical challenges building Parler

Goal (user, 2026-07-01): another SEO-boosting blog post on the *technical challenges*
while building Parler, to gain traction + improve SEO.

## Angle (distinct from the 4 shipped posts)
Debugging war stories: five real bugs that only surfaced past "compiles + passes on my
machine." SEO win: ranks for the exact error/concept searches other Rust/agent devs make
(rustls CryptoProvider panic, tokio-tungstenite wss, auth vs authz, electron restart loop,
async SQLite spawn_blocking). None of the existing posts own this cluster.

## The five war stories (all verified against repo code)
- [ ] 1. WebSocket only broke over TLS — tokio-tungstenite wss + rustls 0.23 dual-provider
      panic (ring vs aws-lc-rs) → ensure_crypto_provider() (parler-connector/src/client.rs)
- [ ] 2. Private hub that wasn't private — key proves identity not authorization; join
      secret, constant-time secret_matches() (parler-hub/src/server.rs, secret.rs)
- [ ] 3. Invite that skipped its own approval gate — minter auto-join self-join bypass; the
      "room already exists + not a member" guard (server.rs ~1254)
- [ ] 4. Crash loop that cooked a MacBook — RestartGate rolling window (desktop/.../restart-gate.ts)
- [ ] 5. One SQLite connection could freeze everyone — blocking I/O on async runtime →
      spawn_blocking for blobs + janitor (server.rs); honest deferral: uploads buffer in RAM

## Build steps
- [ ] Add metadata entry to web/lib/blog.ts POSTS (slug bugs-that-hid-until-production)
- [ ] Create web/components/blog/bugs-that-hid-until-production.tsx (prose primitives)
- [ ] Wire slug->body in web/app/blog/[slug]/page.tsx BODIES + import
- [ ] Create an on-brand SVG cover /public/blog/war-stories.svg
- [ ] Drop prose source docs/blog/bugs-that-hid-until-production.md
- [ ] Voice: no em/en dashes; run humanizer; scan for U+2014/U+2013
- [ ] Verify: cd web && npm run build green

## Review (done 2026-07-02)
- Post shipped: web/components/blog/bugs-that-hid-until-production.tsx (+ POSTS entry, BODIES wire,
  SVG cover web/public/blog/war-stories.svg, docs/blog source, docs/assets copy).
- Distinct angle: debugging war stories → ranks for exact error searches (rustls CryptoProvider
  panic, tokio-tungstenite wss, auth vs authz, electron restart loop, async spawn_blocking).
- All 5 code snippets verified against the real repo before writing.
- House voice: 0 em/en dashes across tsx/ts/svg (md scanned + cleaned too).
- Verified: npm run build GREEN (27/27 static pages); next start smoke — post 200 (correct
  <title>), cover 200 image/svg+xml, index lists it, sitemap+RSS include it, OG card 200 PNG.
- Memory updated: parler-blog-content-strategy (added post #0, refreshed untapped angles).
