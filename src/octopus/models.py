from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum
from time import time
from typing import Any, Mapping
from uuid import uuid4


class NeedType(str, Enum):
    VERIFY = "verify"
    REPRODUCE = "reproduce"
    COMPARE = "compare"
    REMEMBER = "remember"
    FORGET = "forget"
    EXECUTE = "execute"
    RECALL = "recall"
    OBSERVE = "observe"


class Status(str, Enum):
    SATISFIED = "satisfied"
    PARTIAL = "partial"
    FAILED = "failed"
    UNSUPPORTED = "unsupported"


@dataclass(frozen=True)
class Need:
    """A cognitive request with no implementation or tool contract."""

    kind: NeedType
    query: str
    context: Mapping[str, Any] = field(default_factory=dict)
    priority: float = 0.5
    id: str = field(default_factory=lambda: uuid4().hex)
    created_at: float = field(default_factory=time)

    @classmethod
    def verify(cls, query: str, **context: Any) -> "Need":
        return cls(NeedType.VERIFY, query, context)

    @classmethod
    def reproduce(cls, query: str, **context: Any) -> "Need":
        return cls(NeedType.REPRODUCE, query, context)

    @classmethod
    def compare(cls, query: str, **context: Any) -> "Need":
        return cls(NeedType.COMPARE, query, context)

    @classmethod
    def remember(cls, query: str, **context: Any) -> "Need":
        return cls(NeedType.REMEMBER, query, context)

    @classmethod
    def forget(cls, query: str, **context: Any) -> "Need":
        return cls(NeedType.FORGET, query, context)

    @classmethod
    def execute(cls, query: str, **context: Any) -> "Need":
        return cls(NeedType.EXECUTE, query, context)

    @classmethod
    def recall(cls, query: str, **context: Any) -> "Need":
        return cls(NeedType.RECALL, query, context)

    @classmethod
    def observe(cls, query: str, **context: Any) -> "Need":
        return cls(NeedType.OBSERVE, query, context)


@dataclass(frozen=True)
class Evidence:
    source: str
    content: Any
    confidence: float = 1.0
    metadata: Mapping[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class Feed:
    need: Need
    status: Status
    evidence: tuple[Evidence, ...] = ()
    summary: str = ""
    metadata: Mapping[str, Any] = field(default_factory=dict)

    @classmethod
    def satisfied(
        cls,
        need: Need,
        summary: str,
        *,
        evidence: tuple[Evidence, ...] = (),
        **metadata: Any,
    ) -> "Feed":
        return cls(need, Status.SATISFIED, evidence, summary, metadata)

    @classmethod
    def failed(cls, need: Need, summary: str, **metadata: Any) -> "Feed":
        return cls(need, Status.FAILED, (), summary, metadata)

    @classmethod
    def unsupported(cls, need: Need, summary: str = "no tentacle supports this need") -> "Feed":
        return cls(need, Status.UNSUPPORTED, (), summary)


@dataclass(frozen=True)
class Feedback:
    feeds: tuple[Feed, ...]
    summary: str
    status: Status
    created_at: float = field(default_factory=time)
    signals: Mapping[str, Any] = field(default_factory=dict)

    @classmethod
    def from_feeds(cls, feeds: tuple[Feed, ...], **signals: Any) -> "Feedback":
        if not feeds:
            return cls((), "no needs fed", Status.UNSUPPORTED, signals=signals)

        statuses = {feed.status for feed in feeds}
        if statuses == {Status.SATISFIED}:
            status = Status.SATISFIED
        elif Status.FAILED in statuses:
            status = Status.FAILED
        elif Status.UNSUPPORTED in statuses:
            status = Status.UNSUPPORTED
        else:
            status = Status.PARTIAL

        summary = "\n".join(feed.summary for feed in feeds if feed.summary).strip()
        return cls(feeds, summary, status, signals=signals)

