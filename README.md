# Octopus

> Clean agent core: `Need -> Feed -> Feedback`.

## Why

Classic agents bind thought to tool calls. Octopus keeps the brain clean: it asks for cognition, the harness supplies data.

## What it does

- Need: verify, reproduce, compare, remember, forget, execute, recall.
- Feed: tentacles use tools, sub-LLMs, memory, browser, shell, or future arms.
- Feedback: compact evidence returns to the brain.
- Three hearts: heartbeat, memory beat, harness beat.
- Color change: image UI stays optional.

## Quick start

```bash
pip install -e .
```

```python
from octopus import Harness, Need
harness = Harness()
print(harness.feed(Need.remember("tools stay outside the brain")).summary)
```

## Example output

`remembered <id>`

## Proof

Core: `src/octopus`. Tests: `tests`. License: MIT.
