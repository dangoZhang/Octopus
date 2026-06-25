from __future__ import annotations

from collections import defaultdict
from typing import Iterable

from .brain import Brain
from .heart import HarnessBeat, Heartbeat, MemoryBeat
from .memory import MemoryStore
from .models import Evidence, Feedback, Feed, Need, NeedType, Status
from .tentacle import FunctionTentacle, Tentacle


class Harness:
    """Data-driven feed layer between a clean brain and tool tentacles."""

    def __init__(self, *, memory: MemoryStore | None = None):
        self.memory = memory or MemoryStore()
        self._tentacles: list[Tentacle] = []
        self._scores: dict[str, float] = defaultdict(lambda: 1.0)
        self._history: list[Feed] = []
        self.add_tentacle(FunctionTentacle("memory", [NeedType.REMEMBER], self._remember))
        self.add_tentacle(FunctionTentacle("forget", [NeedType.FORGET], self._forget))
        self.add_tentacle(FunctionTentacle("recall", [NeedType.RECALL], self._recall))

    @property
    def routes(self) -> dict[str, float]:
        return dict(self._scores)

    @property
    def history(self) -> tuple[Feed, ...]:
        return tuple(self._history)

    def add_tentacle(self, tentacle: Tentacle) -> None:
        self._tentacles.append(tentacle)
        self._scores[tentacle.name] += 0.0

    def feed(self, needs: Need | Iterable[Need]) -> Feedback:
        batch = (needs,) if isinstance(needs, Need) else tuple(needs)
        feeds = tuple(self._feed_one(need) for need in sorted(batch, key=lambda item: item.priority, reverse=True))
        self._history.extend(feeds)
        return Feedback.from_feeds(feeds, routes=self.routes)

    def evolve(self) -> None:
        for feed in self._history[-100:]:
            name = str(feed.metadata.get("tentacle", "unsupported"))
            if feed.status == Status.SATISFIED:
                self._scores[name] += 0.1
            elif feed.status in {Status.FAILED, Status.UNSUPPORTED}:
                self._scores[name] *= 0.95

    def _feed_one(self, need: Need) -> Feed:
        candidates = [tentacle for tentacle in self._tentacles if tentacle.supports(need)]
        if not candidates:
            return Feed.unsupported(need)

        tentacle = max(candidates, key=lambda candidate: self._scores[candidate.name])
        feed = tentacle.feed(need)
        return Feed(
            need=feed.need,
            status=feed.status,
            evidence=feed.evidence,
            summary=feed.summary,
            metadata={**feed.metadata, "tentacle": tentacle.name},
        )

    def _remember(self, need: Need) -> Feed:
        record = self.memory.remember(need.query, **dict(need.context))
        return Feed.satisfied(
            need,
            f"remembered {record.id}",
            evidence=(Evidence("memory", record.text, metadata={"id": record.id}),),
        )

    def _forget(self, need: Need) -> Feed:
        count = self.memory.forget(need.query)
        return Feed.satisfied(need, f"forgot {count} memories", evidence=(Evidence("memory", count),))

    def _recall(self, need: Need) -> Feed:
        records = self.memory.recall(need.query)
        summary = "\n".join(record.text for record in records) or "nothing recalled"
        return Feed.satisfied(
            need,
            summary,
            evidence=tuple(Evidence("memory", record.text, metadata={"id": record.id}) for record in records),
        )


class Octopus:
    """One-step loop: ask brain for needs, feed them, return feedback."""

    def __init__(self, brain: Brain, harness: Harness | None = None):
        self.brain = brain
        self.harness = harness or Harness()
        self.hearts = (
            Heartbeat(),
            MemoryBeat(self.harness.memory),
            HarnessBeat(self.harness),
        )

    def pulse(self, feedback: Feedback | None = None) -> Feedback:
        for heart in self.hearts:
            heart.beat()
        needs = tuple(self.brain.needs(feedback))
        result = self.harness.feed(needs)
        self.brain.needs(result)
        return result

