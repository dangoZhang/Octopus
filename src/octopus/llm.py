from __future__ import annotations

import json
import os
import urllib.request
from dataclasses import dataclass


@dataclass(frozen=True)
class OpenAICompatibleLLM:
    """TextLLM adapter for OpenAI-compatible chat completions endpoints."""

    model: str
    api_key: str | None = None
    base_url: str = "https://api.openai.com/v1"
    timeout: float = 60.0

    def complete(self, system: str, user: str) -> str:
        payload = {
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
        }
        request = urllib.request.Request(
            f"{self.base_url.rstrip('/')}/chat/completions",
            data=json.dumps(payload).encode("utf-8"),
            headers=self._headers(),
            method="POST",
        )
        with urllib.request.urlopen(request, timeout=self.timeout) as response:
            data = json.loads(response.read().decode("utf-8"))
        try:
            return str(data["choices"][0]["message"]["content"])
        except (KeyError, IndexError, TypeError) as exc:
            raise ValueError("chat completion response missing choices[0].message.content") from exc

    def _headers(self) -> dict[str, str]:
        headers = {"Content-Type": "application/json"}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
        return headers


def llm_from_env(prefix: str = "OCTOPUS_LLM") -> OpenAICompatibleLLM:
    model = os.getenv(f"{prefix}_MODEL")
    if not model:
        raise ValueError(f"{prefix}_MODEL is required")
    return OpenAICompatibleLLM(
        model=model,
        api_key=os.getenv(f"{prefix}_API_KEY"),
        base_url=os.getenv(f"{prefix}_BASE_URL", "https://api.openai.com/v1"),
        timeout=float(os.getenv(f"{prefix}_TIMEOUT", "60")),
    )
