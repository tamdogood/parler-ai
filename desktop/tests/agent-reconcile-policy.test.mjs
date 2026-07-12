import assert from "node:assert/strict";
import test from "node:test";
import { hostsNeedingConnection } from "../src/main/agent-reconcile-policy.ts";

function host(id, overrides = {}) {
  return {
    id,
    name: id,
    installed: true,
    connected: false,
    connectedTarget: null,
    method: "config",
    ...overrides,
  };
}

test("only returns installed hosts that are missing or pointed at another hub", () => {
  const hosts = [
    host("missing"),
    host("wrong-hub", { connected: true, connectedTarget: "public" }),
    host("ready", { connected: true, connectedTarget: "local" }),
    host("not-installed", { installed: false }),
  ];

  assert.deepEqual(
    hostsNeedingConnection(hosts, "local").map((candidate) => candidate.id),
    ["missing", "wrong-hub"],
  );
});

test("respects the selected shared hub", () => {
  const hosts = [
    host("local", { connected: true, connectedTarget: "local" }),
    host("shared", { connected: true, connectedTarget: "public" }),
  ];

  assert.deepEqual(
    hostsNeedingConnection(hosts, "public").map((candidate) => candidate.id),
    ["local"],
  );
});
