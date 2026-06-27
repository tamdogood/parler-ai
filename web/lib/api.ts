import type { DirectoryEntry, HubSummary, Scope } from "./types";

/** Base URL of the parler-hub REST API. */
export const HUB_API =
  process.env.NEXT_PUBLIC_HUB_API?.replace(/\/$/, "") || "http://127.0.0.1:7070";

const TOKEN_KEY = "parler.directoryToken";

export function getDirectoryToken(): string | null {
  if (typeof window === "undefined") return null;
  return window.localStorage.getItem(TOKEN_KEY);
}

export function setDirectoryToken(token: string | null) {
  if (typeof window === "undefined") return;
  if (token) window.localStorage.setItem(TOKEN_KEY, token);
  else window.localStorage.removeItem(TOKEN_KEY);
}

function authHeaders(): HeadersInit {
  const t = getDirectoryToken();
  return t ? { Authorization: `Bearer ${t}` } : {};
}

export class HubError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}

async function getJson<T>(path: string, withAuth = false): Promise<T> {
  const res = await fetch(`${HUB_API}${path}`, {
    headers: withAuth ? authHeaders() : undefined,
    cache: "no-store",
  });
  if (!res.ok) {
    let msg = `${res.status} ${res.statusText}`;
    try {
      const body = (await res.json()) as { error?: string };
      if (body.error) msg = body.error;
    } catch {
      /* non-JSON error body */
    }
    throw new HubError(msg, res.status);
  }
  return (await res.json()) as T;
}

export function fetchHub(): Promise<HubSummary> {
  return getJson<HubSummary>("/api/hub");
}

export interface DiscoverParams {
  scope: Scope;
  q?: string;
  tag?: string;
  skill?: string;
  status?: string;
}

export function fetchDirectory(params: DiscoverParams): Promise<DirectoryEntry[]> {
  const qs = new URLSearchParams();
  qs.set("scope", params.scope);
  if (params.q) qs.set("q", params.q);
  if (params.tag) qs.set("tag", params.tag);
  if (params.skill) qs.set("skill", params.skill);
  if (params.status) qs.set("status", params.status);
  // Hub scope may require a directory bearer token on a private hub.
  return getJson<DirectoryEntry[]>(`/api/directory?${qs.toString()}`, params.scope === "hub");
}
