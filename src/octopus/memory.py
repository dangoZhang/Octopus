from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from time import time
from typing import Any
from uuid import uuid4


@dataclass
class MemoryRecord:
    text: str
    metadata: dict[str, Any] = field(default_factory=dict)
    id: str = field(default_factory=lambda: uuid4().hex)
    created_at: float = field(default_factory=time)
    weight: float = 1.0

    def to_dict(self) -> dict[str, Any]:
        return {
            "id": self.id,
            "text": self.text,
            "metadata": self.metadata,
            "created_at": self.created_at,
            "weight": self.weight,
        }

    @classmethod
    def from_dict(cls, data: dict[str, Any]) -> "MemoryRecord":
        return cls(
            id=str(data["id"]),
            text=str(data["text"]),
            metadata=dict(data.get("metadata", {})),
            created_at=float(data.get("created_at", time())),
            weight=float(data.get("weight", 1.0)),
        )


class MemoryStore:
    def __init__(self, records: tuple[MemoryRecord, ...] = ()):
        self._records: dict[str, MemoryRecord] = {record.id: record for record in records}

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
            matches = sum(1 for word in words if word in haystack)
            if not words or matches:
                return matches + record.weight * 0.01
            return 0.0

        records = sorted(self._records.values(), key=score, reverse=True)
        recalled = tuple(record for record in records[:limit] if score(record) > 0)
        for record in recalled:
            record.weight += 0.1
        return recalled

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

    def snapshot(self) -> dict[str, Any]:
        records = sorted(self._records.values(), key=lambda record: record.created_at)
        return {"records": [record.to_dict() for record in records]}

    @classmethod
    def from_snapshot(cls, data: dict[str, Any]) -> "MemoryStore":
        records = tuple(MemoryRecord.from_dict(item) for item in data.get("records", []))
        return cls(records)

    def save(self, path: str | Path) -> None:
        target = Path(path)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(json.dumps(self.snapshot(), ensure_ascii=False, indent=2), encoding="utf-8")

    @classmethod
    def load(cls, path: str | Path) -> "MemoryStore":
        source = Path(path)
        if not source.exists():
            return cls()
        return cls.from_snapshot(json.loads(source.read_text(encoding="utf-8")))
