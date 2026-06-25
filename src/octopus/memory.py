from __future__ import annotations

from dataclasses import dataclass, field
from time import time
from typing import Any
from uuid import uuid4


@dataclass(frozen=True)
class MemoryRecord:
    text: str
    metadata: dict[str, Any] = field(default_factory=dict)
    id: str = field(default_factory=lambda: uuid4().hex)
    created_at: float = field(default_factory=time)
    weight: float = 1.0


class MemoryStore:
    def __init__(self):
        self._records: dict[str, MemoryRecord] = {}

    def remember(self, text: str, **metadata: Any) -> MemoryRecord:
        record = MemoryRecord(text=text, metadata=metadata)
        self._records[record.id] = record
        return record

    def forget(self, query: str) -> int:
        matched = [
            record_id
            for record_id, record in self._records.items()
            if query.lower() in record.text.lower()
            or any(query.lower() in str(value).lower() for value in record.metadata.values())
        ]
        for record_id in matched:
            del self._records[record_id]
        return len(matched)

    def recall(self, query: str, limit: int = 5) -> tuple[MemoryRecord, ...]:
        words = {word.lower() for word in query.split() if word.strip()}

        def score(record: MemoryRecord) -> float:
            haystack = f"{record.text} {record.metadata}".lower()
            return sum(1 for word in words if word in haystack) + record.weight * 0.01

        records = sorted(self._records.values(), key=score, reverse=True)
        return tuple(record for record in records[:limit] if score(record) > 0)

    def compact(self, *, keep: int = 200) -> int:
        if len(self._records) <= keep:
            return 0
        records = sorted(self._records.values(), key=lambda record: (record.weight, record.created_at))
        drop = records[: len(self._records) - keep]
        for record in drop:
            del self._records[record.id]
        return len(drop)

    def __len__(self) -> int:
        return len(self._records)

