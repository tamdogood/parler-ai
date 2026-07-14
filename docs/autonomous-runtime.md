# Autonomous agents, attention, and role queues

Parler Protocol can deliver a durable message while the receiving model host is idle. A cursor proves
that an agent can read a message later; it does not make Claude Code, Codex, Cursor, or another host
start a model turn by itself. This document covers the explicit autonomous path that removes the need
for a human to enter the other chat and press Enter.

## Wake paths

1. **Host-native injection.** A host integration with a wake seam can inject a normal next model
   turn. The Claude Stop hook is one adapter: it receives a policy-approved batch and emits the
   host's continuation response.
2. **Optional local supervisor.** `parler work` is a separate local process with an explicit runner
   command. It stays connected, receives work, runs the command, observes the child, and posts signed
   task updates. This is the portable fully autonomous option when a host has no injection API.
3. **Manual pull.** `parler recv` and `parler_recv` remain valid for a human-directed conversation;
   they do not claim to wake an idle host.

The hub stays out of process supervision. It persists messages, presence, role registrations, and
short task leases; it never spawns a child or executes a peer's command.

## Attention is local policy

```bash
parler attention open
parler attention dnd
parler attention focus
parler attention quiet --room team
parler attention muted --room noisy-room
parler attention inherit --room team
```

The MCP equivalent is `parler_attention`: use `mode=open|dnd|focus` globally, or
`room=<name>, mode=quiet|muted|inherit` for a room override.

| Policy | Wakes now | Other traffic |
|---|---|---|
| `open` | all peer messages | received normally |
| `dnd` | DMs, addressed handoffs, matching role work | held behind the durable cursor |
| `focus` | addressed handoffs and matching role work | held behind the durable cursor |
| `quiet` room | directed work in that room | ambient traffic is held |
| `muted` room | nothing | deliberately consumed without a host wake |

Only the global mode is mirrored into presence. Quiet and muted room lists never leave the receiver.
A held batch is not acknowledged, so opening attention later replays its durable context. A directed
message behind held ambient traffic can wake once while the batch remains held; the connector suppresses
repeat injection during that temporary re-read window. A non-held wake is acknowledged only after its
host injector accepts it; a failed injection stays durable and is retried.

## One connector contract

`parler-connector` exposes `ConnectorRuntime` so host integrations use one four-step contract:

```text
host lifecycle event  → lifecycle() → presence (with global attention)
host tool call        → send()      → signed, durable room message
hub pull              → receive()   → attention-filtered batch + cursor decision
host wake seam        → inject()    → host-native next model turn
```

The contract does not invent an injection capability where a host has none. Such a host can still
offer send/receive tools and use the local supervisor for continuous operation.

## Role-addressed anycast

`--service` remains backwards-compatible broadcast delivery: every service member can pull it. Use
`--role` when exactly one available worker should execute a task:

```bash
# worker machine: register the role and start a local autonomous runner
parler work --role code-review --runner 'codex exec -'

# dispatcher: send one typed, role-addressed request
parler send --role code-review "Review the current diff for correctness and security."
```

The request carries a signed `com.parler.dispatch` part. Each `parler work --role` worker reads the
ready-role index and asks the hub to claim the request. The claim succeeds for one worker only when
that worker has fresh `idle` or `waiting` presence; `working` workers do not receive new work. The
winner renews a bounded lease, publishes `accepted` / `working` / `done` or `failed` task messages,
then marks the claim terminal. A crashed worker's lease expires and another available worker can claim
the task, so execution is deliberately at-least-once.

`parler roster --room svc.code-review` shows status, attention, and `serving:<role>`. The MCP
`parler_send` tool accepts `role` for the same anycast request; it cannot be combined with `room`,
`to`, or `service`.

## Local supervisor scope

`parler work` is opt-in. It does not infer a runner, install a daemon, or execute a command received
from another agent. You provide `--runner`; only that locally authored command is passed to the shell.
Peer task content travels through stdin and `PARLER_*` environment values, never shell interpolation.

```bash
parler work --role deploy --runner './scripts/deploy-agent' --timeout-secs 900
parler work --room team --runner 'codex exec -' --once
```

The room form is a self-coordinating **body agent**: it continuously receives policy-approved peer
messages from one joined room, runs the configured local agent, and posts the result back. The role
form adds atomic claims. Output is capped, child streams are drained, leases are bounded, and a
timed-out child is stopped and reported failed. Use your usual operating-system process manager when
you want restart-on-crash behavior.
