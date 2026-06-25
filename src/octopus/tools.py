from __future__ import annotations

import shlex
import subprocess
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any, Protocol

from .models import Evidence, Need, NeedType, Status


@dataclass(frozen=True)
class ToolResult:
    status: Status
    output: Any
    evidence: tuple[Evidence, ...] = ()
    metadata: dict[str, Any] = field(default_factory=dict)

    @classmethod
    def satisfied(cls, tool_name: str, output: Any, **metadata: Any) -> "ToolResult":
        return cls(Status.SATISFIED, output, (Evidence(tool_name, output),), metadata)

    @classmethod
    def failed(cls, output: Any, **metadata: Any) -> "ToolResult":
        return cls(Status.FAILED, output, (), metadata)


class Tool(Protocol):
    name: str
    description: str

    def supports(self, need: Need) -> bool:
        ...

    def run(self, need: Need) -> ToolResult:
        ...


class FunctionTool:
    def __init__(
        self,
        name: str,
        description: str,
        kinds: tuple[NeedType, ...],
        handler: Callable[[Need], Any],
    ):
        self.name = name
        self.description = description
        self._kinds = set(kinds)
        self._handler = handler

    def supports(self, need: Need) -> bool:
        return need.kind in self._kinds

    def run(self, need: Need) -> ToolResult:
        try:
            return ToolResult.satisfied(self.name, self._handler(need))
        except Exception as exc:  # pragma: no cover - keeps tool failure in-band.
            return ToolResult.failed(f"{self.name} failed: {exc}", tool=self.name)


class ShellTool:
    """Explicitly allowlisted local command tool."""

    name = "shell"
    description = "runs allowlisted local commands"

    def __init__(self, allowed: tuple[str, ...], *, timeout: float = 30.0):
        self.allowed = allowed
        self.timeout = timeout

    def supports(self, need: Need) -> bool:
        return need.kind in {NeedType.EXECUTE, NeedType.REPRODUCE, NeedType.VERIFY}

    def run(self, need: Need) -> ToolResult:
        command = str(need.context.get("command") or need.query)
        argv = shlex.split(command)
        if not argv:
            return ToolResult.failed("empty command", tool=self.name)
        if argv[0] not in self.allowed:
            return ToolResult.failed(f"command not allowed: {argv[0]}", tool=self.name)

        completed = subprocess.run(
            argv,
            check=False,
            capture_output=True,
            text=True,
            timeout=self.timeout,
        )
        output = (completed.stdout or completed.stderr).strip()
        status = Status.SATISFIED if completed.returncode == 0 else Status.FAILED
        evidence = (Evidence(self.name, output, metadata={"returncode": completed.returncode}),)
        return ToolResult(status, output, evidence, {"tool": self.name, "returncode": completed.returncode})

