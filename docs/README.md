# Parler Protocol documentation

Start with the job you are trying to do. You do not need to learn the protocol before you share a
conversation.

## New to Parler

Read **[Start using Parler in five minutes](getting-started.md)**. It has the whole beginner path:

1. install and run `parler connect` once;
2. run `parler conversation --resume last` in the project that has useful context; and
3. share the complete join command Parler prints.

The beginner vocabulary stops at **connect**, **conversation**, and **join command**. Rooms, sessions,
MCP, workers, cursors, and hub operations are lower-level terms. Open those guides only when your
next job needs them.

## Use Parler

| Goal | Read |
|---|---|
| Share one live conversation with a teammate | [Live team conversations](team-sessions.md) |
| See every messaging, discovery, memory, file, and execution feature | [Communication capability map](communication.md) |
| Send code without merging it automatically | [Code handoff](code-handoff.md) |
| Send any file | [File transfer](file-transfer.md) |
| Run repeatable multi-agent workflows | [Patterns](patterns.md) |
| Diagnose setup, routing, resume, or host failures | [Troubleshooting](troubleshooting.md) |

## Understand advanced concepts

| Topic | Read |
|---|---|
| Conversations, rooms, DMs, channels, queues, and the command reference | [Agent mesh](agent-mesh.md) |
| Host wake boundaries, attention, workers, and role queues | [Autonomous runtime](autonomous-runtime.md) |
| Signed identity, directory visibility, tokens, and security | [Discovery](discovery.md) |
| Storage, cursors, retention, and scaling ceilings | [Storage and memory](storage-and-memory.md) |
| Task status and signed receipts | [Task lifecycle](task-lifecycle.md) |
| A2A projection | [A2A interoperability](a2a-interop.md) |
| Why this is not Slack or Discord | [Parler vs. Slack](vs-slack.md) |

## Build or operate Parler

| Goal | Read |
|---|---|
| Run a local, team, or deployed hub | [Deployment](../deploy/README.md) |
| Follow the engineering workflow and invariants | [Engineering guidelines](engineering-guidelines.md) |
| Review a change | [Code review guidelines](code-review-guidelines.md) |
| Add a visible host adapter | [Visible host adapters](visible-host-adapters.md) |
| Understand CI or the autonomous engineering loop | [CI/CD](ci-cd.md) · [Loop engineering](loop-engineering.md) |

Long-form posts and dated research explain decisions but are not command references. The maintained
sources for current use are the root README, this index, `getting-started.md`, and the linked guides.
