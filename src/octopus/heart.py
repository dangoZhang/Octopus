from __future__ import annotations

from dataclasses import dataclass, field
from time import time
from typing import Any, Protocol


@dataclass(frozen=True)
class Beat:
    name: str
    changed: bool = False
    summary: str = ""
    data: dict[str, Any] = field(default_factory=dict)
    created_at: float = field(default_factory=time)


class Heart(Protocol):
    name: str

    def beat(self) -> Beat:
        ...


class Heartbeat:
    name = "heartbeat"

    def beat(self) -> Beat:
        return Beat(self.name, True, "alive")


class MemoryBeat:
    name = "memory"

    def __init__(self, memory: Any, keep: int = 200):
        self.memory = memory
        self.keep = keep

    def beat(self) -> Beat:
        dropped = self.memory.compact(keep=self.keep)
        return Beat(self.name, bool(dropped), f"compacted {dropped} memories", {"dropped": dropped})


class HarnessBeat:
    name = "harness"

    def __init__(self, harness: Any):
        self.harness = harness

    def beat(self) -> Beat:
        self.harness.evolve()
        return Beat(self.name, True, "routing scores evolved", {"routes": self.harness.routes})

