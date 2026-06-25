"""Parler plugin for Hermes — joins the gateway to the Parler mesh.

Registers three things on the Hermes plugin context:
  - a **gateway platform adapter** (``parler``) — inbound wake/drive + outbound reply routing,
  - **lifecycle hooks** → Parler presence (over the connector's control socket),
  - the **parler_* tools** — proactive mesh actions for the agent (full shared parity).

All three talk to the Rust sidecar (which owns the mesh endpoint) over the sockets the launcher set
in the environment. Two run modes:
  - **Managed** — the Parler launcher spawns this gateway with PARLER_BRIDGE_SOCKET (and the control
    socket + tools file) already set; we just register against them.
  - **Standalone** — a user's own ``hermes`` with the plugin installed: PARLER_SPACE / PARLER_NAME /
    PARLER_SERVERS are set but no bridge socket, so we spawn the bundled sidecar ourselves, derive
    the socket/file paths, and register against those.
"""
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import time
from pathlib import Path
from typing import Any

from . import hooks
from .tools import register_tools


def _is_managed() -> bool:
    """Managed launches preset the bridge socket; its presence means the sidecar already exists."""
    return bool(os.environ.get("PARLER_BRIDGE_SOCKET"))


def _have_identity() -> bool:
    """A name, a join link, or an agent file is the explicit opt-in. A PARLER_LINK carries space +
    server + auth on its own, so a peer can join with just the link (+ an optional PARLER_NAME)."""
    return bool(
        os.environ.get("PARLER_NAME")
        or os.environ.get("PARLER_LINK")
        or os.environ.get("PARLER_AGENT_FILE")
    )


def _check_requirements() -> bool:
    """Enable the parler platform when we can reach (or bootstrap) a sidecar: a preset bridge socket
    (managed) or enough identity to spawn one (standalone)."""
    return _is_managed() or _have_identity()


def _resolve_sidecar() -> str:
    """Locate the parler-connect-hermes sidecar binary. Honors a PARLER_SIDECAR_BIN override;
    otherwise resolves ``parler-connect-hermes`` on PATH. Throws if absent — no silent fallback."""
    override = os.environ.get("PARLER_SIDECAR_BIN")
    if override:
        if not Path(override).is_file():
            raise RuntimeError(f"PARLER_SIDECAR_BIN={override} does not exist")
        return override
    on_path = shutil.which("parler-connect-hermes")
    if on_path:
        return on_path
    raise RuntimeError(
        "parler-connect-hermes sidecar not found on PATH — build it (cargo build) or set "
        "PARLER_SIDECAR_BIN"
    )


def _bootstrap_standalone_sidecar() -> None:
    """Standalone mode: spawn the sidecar binary and wait until it has published the bridge socket +
    tools file. Sets the three path env vars first so the sidecar and the rest of this registration
    agree on them."""
    run_dir = Path(tempfile.mkdtemp(prefix="parler-hermes-"))
    name = os.environ.get("PARLER_NAME") or "hermes"
    space = os.environ.get("PARLER_SPACE") or "(from PARLER_LINK)"
    os.environ.setdefault("PARLER_BRIDGE_SOCKET", str(run_dir / "bridge.sock"))
    os.environ.setdefault("PARLER_CONTROL_SOCKET", str(run_dir / "control.sock"))
    os.environ.setdefault("PARLER_TOOLS_FILE", str(run_dir / "parler-tools.json"))

    sidecar = _resolve_sidecar()
    env = os.environ.copy()
    # Tie the sidecar's life to THIS process (the gateway that loaded the plugin); it watches this pid.
    env["PARLER_PARENT_PID"] = str(os.getpid())
    env.setdefault("PARLER_STANDALONE", "1")
    subprocess.Popen(  # noqa: S603 — trusted bundled asset
        [sidecar],
        env=env,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )

    tools_file = Path(os.environ["PARLER_TOOLS_FILE"])
    bridge_sock = Path(os.environ["PARLER_BRIDGE_SOCKET"])
    deadline = time.monotonic() + 20.0
    while time.monotonic() < deadline:
        if tools_file.is_file() and bridge_sock.exists():
            return
        time.sleep(0.1)
    raise RuntimeError(
        f"parler sidecar did not come up within 20s for {name}@{space} "
        f"(tools={tools_file.is_file()}, bridge={bridge_sock.exists()})"
    )


def register(ctx: Any) -> None:
    if not _is_managed():
        if not _have_identity():
            return  # not a parler gateway — nothing to wire up
        _bootstrap_standalone_sidecar()

    # Gateway platform adapter (imported lazily so a non-gateway context still loads tools/hooks).
    from .adapter import ParlerAdapter

    # Parler mesh peers are already authorized by the NATS JWT, and an autonomous gateway has no
    # operator to approve a pairing code or pick a home channel — suppress both first-contact prompts
    # for the parler platform ONLY, so a standalone user's other platforms keep their access control.
    os.environ.setdefault("PARLER_ALLOW_ALL_USERS", "true")
    os.environ.setdefault("PARLER_HOME_CHANNEL", "mesh")

    ctx.register_platform(
        name="parler",
        label="Parler",
        adapter_factory=lambda cfg: ParlerAdapter(cfg),
        check_fn=_check_requirements,
        allowed_users_env="PARLER_ALLOWED_USERS",
        allow_all_env="PARLER_ALLOW_ALL_USERS",
        platform_hint=(
            "You are coordinating with peer agents on the Parler mesh. Your reply is delivered "
            "automatically back to whoever messaged you; use parler_* tools only to reach OTHER "
            "peers/channels or report status."
        ),
        emoji="🔗",
        max_message_length=8000,
    )

    # Lifecycle hooks → presence.
    ctx.register_hook("on_session_start", hooks.on_session_start)
    ctx.register_hook("pre_llm_call", hooks.pre_llm_call)
    ctx.register_hook("pre_tool_call", hooks.pre_tool_call)
    ctx.register_hook("post_llm_call", hooks.post_llm_call)
    ctx.register_hook("on_session_end", hooks.on_session_end)

    # Proactive mesh tools (declared from the descriptors the sidecar wrote to PARLER_TOOLS_FILE).
    register_tools(ctx)
