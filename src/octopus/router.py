from __future__ import annotations

from dataclasses import dataclass

from .models import Feed, Need, Status


@dataclass(frozen=True)
class RouteDecision:
    tentacle: str
    score: float
    reason: str


class RouteBook:
    """Small data-driven router for harness self-evolution."""

    def __init__(self, *, base_score: float = 1.0):
        self.base_score = base_score
        self._scores: dict[tuple[str, str], float] = {}

    def choose(self, need: Need, tentacle_names: tuple[str, ...]) -> RouteDecision:
        scored = [(name, self.score(need, name)) for name in tentacle_names]
        name, score = max(scored, key=lambda item: item[1])
        return RouteDecision(name, score, f"{need.kind.value}:{name}={score:.2f}")

    def score(self, need: Need, tentacle_name: str) -> float:
        return self._scores.get((need.kind.value, tentacle_name), self.base_score)

    def learn(self, feed: Feed) -> None:
        name = str(feed.metadata.get("tentacle", ""))
        if not name:
            return

        key = (feed.need.kind.value, name)
        current = self._scores.get(key, self.base_score)
        confidence = self._confidence(feed)
        if feed.status == Status.SATISFIED:
            next_score = current + 0.2 * confidence
        elif feed.status == Status.PARTIAL:
            next_score = current + 0.05 * confidence
        elif feed.status == Status.FAILED:
            next_score = current * 0.7
        else:
            next_score = current * 0.9
        self._scores[key] = min(10.0, max(0.05, next_score))

    def decay(self, rate: float = 0.995) -> None:
        self._scores = {key: max(0.05, value * rate) for key, value in self._scores.items()}

    def snapshot(self) -> dict[str, float]:
        return {f"{kind}:{name}": value for (kind, name), value in sorted(self._scores.items())}

    def _confidence(self, feed: Feed) -> float:
        if not feed.evidence:
            return 0.5
        return sum(item.confidence for item in feed.evidence) / len(feed.evidence)

