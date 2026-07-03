# Wiring a post into the Next.js site

Four edits and one new file. Do them in this order. Paths are from the repo root; the app
lives in `web/`.

## 1. Register metadata in `web/lib/blog.ts`

Add an entry to the top of the `POSTS` array (newest first). All SEO surfaces (sitemap,
RSS, per-post OpenGraph + Twitter cards, and the `BlogPosting` JSON-LD) auto-derive from
this, so you do not wire them separately.

```ts
{
  slug: "your-slug",                 // kebab-case, matches component + docs file
  title: "Sentence case, no dashes",
  dek: "One-to-two sentence standfirst. This is your meta description, so front-load the
        keyword phrase and make it read like a promise, not a summary.",
  date: "2026-07-DD",                // ISO, drives <time> + sort order
  dateLabel: "July D, 2026",         // human label
  readingTime: "N min read",
  author: "Your Name",
  tags: ["Primary keyword", "Secondary", "..."],  // these become JSON-LD keywords
  cover: "/blog/your-slug.svg",      // lives in web/public/blog/
},
```

## 2. Write the body component: `web/components/blog/<slug>.tsx`

Export a named React component. **Use only the prose primitives** from
`components/blog/prose.tsx` so every post renders identically. Available primitives:

`ArticleH2`, `ArticleH3`, `P`, `Lead`, `UL`, `LI`, `Em`, `InlineCode`, `A`, `Divider`,
`CodeBlock`, `Figure`, `Callout`, `RefTable`.

Give H2s an `id` so they can be deep-linked. Use `A` (not raw `<a>`) for links, and link to
1-2 related posts by `/blog/<slug>` for internal SEO. Shape:

```tsx
import { Lead, P, ArticleH2, Em, A, InlineCode, CodeBlock, Callout, UL, LI } from "./prose";

export function YourPostName() {
  return (
    <>
      <Lead>The opening that names a concrete, specific problem.</Lead>
      <ArticleH2 id="the-problem">The problem, plainly</ArticleH2>
      <P>...</P>
      <CodeBlock language="rust">{`// real code from crates/, quoted accurately`}</CodeBlock>
      {/* more sections */}
    </>
  );
}
```

Copy the structure of an existing component (e.g. `how-agents-hand-off-code.tsx`) rather
than inventing a new layout.

## 3. Register the body in `web/app/blog/[slug]/page.tsx`

Two edits in this file:

1. Import the component at the top with the other `@/components/blog/*` imports.
2. Add a line to the `BODIES` map: `"your-slug": <YourPostName />,` (put it at the top to
   match newest-first order).

The page falls back to `notFound()` if either the `POSTS` entry or the `BODIES` line is
missing, so both are required.

## 4. Drop the prose source: `docs/blog/<slug>.md`

The plain-markdown draft you wrote and humanized lives here permanently. It's the repo
pattern and the source of truth for the prose; the `.tsx` is the rendered form.

## 5. Cover image: `web/public/blog/<slug>.svg`

SVG is fine and is served via a plain `<img>` (no rasterizer). On-brand palette: black
`#000`, electric-blue `#3b9eff`, violet `#9281f7`, green `#3ecf8e`, graphite hairlines. If
you can't produce a good one, reuse an existing cover rather than shipping something ugly.

## Verify

From `web/`:

```
npm run build
```

Must be green. Then `next start` and confirm 200s for: the post page `/blog/<slug>`, the
cover, the `/blog` index (new card shows), `/sitemap.xml`, and `/rss.xml` (new slug
present). Re-run `bash .claude/skills/write-blog/check.sh docs/blog/<slug>.md`.
