import type { HubTarget, McpHost } from "../shared/types";

/**
 * Hosts that background automation may safely wire. Already-correct hosts are deliberately omitted
 * so a periodic scan does not keep rewriting user configuration files or asking agents to restart.
 */
export function hostsNeedingConnection(hosts: McpHost[], target: HubTarget): McpHost[] {
  return hosts.filter((host) => host.installed && (!host.connected || host.connectedTarget !== target));
}
