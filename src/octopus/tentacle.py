from __future__ import annotations

from collections.abc import Callable
from typing import Iterable, Protocol

from .models import Evidence, Feed, Need, NeedType, Status


class Tentacle(Protocol):
    name: str

    def supports(self, need: Need) -> bool:
        ...

    def feed(self, need: Need) -> Feed:
        ...


class TentacleBrain(Protocol):
    """Tool-side intelligence that turns needs into concrete tool work."""

    def handle(self, need: Need) -> Feed:
        ...


class FunctionTentacle:
    def __init__(
        self,
        name: str,
        supports: Iterable[NeedType],
        handler: Callable[[Need], str | Feed | Evidence],
    ):
        self.name = name
        self._supports = set(supports)
        self._handler = handler

    def supports(self, need: Need) -> bool:
        return need.kind in self._supports

    def feed(self, need: Need) -> Feed:
        try:
            result = self._handler(need)
        except Exception as exc:  # pragma: no cover - preserves feedback contract.
            return Feed.failed(need, f"{self.name} failed: {exc}", tentacle=self.name)

        if isinstance(result, Feed):
            return result
        if isinstance(result, Evidence):
            return Feed(need, Status.SATISFIED, (result,), str(result.content), {"tentacle": self.name})
        return Feed.satisfied(
            need,
            str(result),
            evidence=(Evidence(self.name, result),),
            tentacle=self.name,
        )


class SmartTentacle:
    """A tentacle with its own brain, so the clean brain stays tool-free."""

    def __init__(self, name: str, supports: Iterable[NeedType], brain: TentacleBrain):
        self.name = name
        self._supports = set(supports)
        self._brain = brain

    def supports(self, need: Need) -> bool:
        return need.kind in self._supports

    def feed(self, need: Need) -> Feed:
        feed = self._brain.handle(need)
        return Feed(
            need=feed.need,
            status=feed.status,
            evidence=feed.evidence,
            summary=feed.summary,
            metadata={**feed.metadata, "tentacle": self.name},
        )

