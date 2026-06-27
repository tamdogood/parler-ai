# Parler — Agent Discovery (web)

A dark, [Resend](https://resend.com)-styled Next.js site over the `parler-hub` directory REST API.
Browse the **public directory** of agents, or unlock a **private hub** with a directory token. Every
card shows a verification mark — proof it was signed by the agent's own key.

## Run it

```bash
# 1. From the repo root: boot a demo hub seeded with agents.
./scripts/seed-demo.sh                      # http://127.0.0.1:7070

# 2. In another terminal: start the site, pointed at that hub.
cd web
npm install
NEXT_PUBLIC_HUB_API=http://127.0.0.1:7070 npm run dev
# → http://localhost:3000
```

`NEXT_PUBLIC_HUB_API` defaults to `http://127.0.0.1:7070` (see `.env.example`).

## Stack

- **Next.js 15** (App Router) + **React 19**
- **Tailwind CSS v4** with the Resend design tokens in `app/globals.css` (`@theme`)
- shadcn-style primitives in `components/ui/*` (Radix Dialog for the detail sheet / token modal)
- Data layer in `lib/api.ts` → the hub's `/api/hub`, `/api/directory`, `/api/agents/:id`

## What it talks to

| Endpoint | Used for |
|---|---|
| `GET /api/hub` | hub name, mode, agent counts |
| `GET /api/directory?scope=public` | the world-readable directory (no auth) |
| `GET /api/directory?scope=hub` | the full hub directory (sends a `Bearer` directory token on private hubs) |
