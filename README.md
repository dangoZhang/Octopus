# Octopus

> Clean agent core with smart arms: `Need -> Feed -> Feedback`.

## Why

Classic agents bind thought to tool calls. Octopus keeps the brain clean: it asks for cognition, the harness supplies data, and tentacles think during execution.

## What it does

- Need: verify, reproduce, compare, remember, forget, execute, recall.
- Feed: thinking tentacles use tools, sub-LLMs, memory, browser, shell, or future arms.
- Feedback: compact evidence returns to the brain.
- Three hearts: heartbeat, memory beat, harness route beat.

## Quick start

```bash
pip install -e .
octopus need remember "tools stay outside the brain"
```

```python
from octopus import FunctionTool, Harness, Need, NeedType, PlanningTentacleBrain, SmartTentacle
harness = Harness()
tool = FunctionTool("verify", "checks claims", (NeedType.VERIFY,), lambda need: "verified")
harness.add_tentacle(SmartTentacle("research", [NeedType.VERIFY], PlanningTentacleBrain((tool,))))
print(harness.feed(Need.verify("Need does not name tools.")).summary)
```

## Example output

`verified`

## Proof

Core: `src/octopus`. Docs: `docs`. Tests: `tests`. License: MIT.
