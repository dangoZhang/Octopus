from __future__ import annotations

from typing import Protocol

from .models import Feedback


class Chromatophore(Protocol):
    """Optional image/UI surface. It observes feedback without owning the core."""

    def render(self, feedback: Feedback) -> bytes:
        ...

