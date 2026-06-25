from __future__ import annotations

from typing import Iterable, Protocol

from .models import Feedback, Need


class Brain(Protocol):
    """Brain implementations emit needs and receive compact feedback."""

    def needs(self, feedback: Feedback | None = None) -> Iterable[Need]:
        ...


class StaticBrain:
    """Tiny brain for demos and tests."""

    def __init__(self, needs: Iterable[Need]):
        self._needs = tuple(needs)
        self.feedback: tuple[Feedback, ...] = ()

    def needs(self, feedback: Feedback | None = None) -> Iterable[Need]:
        if feedback is not None:
            self.feedback += (feedback,)
            return ()
        return self._needs

