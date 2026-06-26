# Octopus 🐙

> Tools that think on their own, with a brain that stays clean.

Octopus starts from a small bet: an agent should not carry every tool, memory rule, and execution detail inside its main thought loop. The model should ask for the cognition it needs. The environment should learn how to supply it.

That gives Octopus a different shape from the usual agent loop. The main brain stays focused on the goal and the next cognitive demand. The tentacles own the messy part: tool metadata, executable code, local permissions, runtime adapters, and feedback. When a tool runs, a tentacle can plan, act, inspect evidence, and return a compact result.

The result is a lighter brain and smarter tools.

## Why It Feels Different

- The main context stays small: goal, memory, request, result.
- Tools are productized as editable tentacles: prompt, metadata, code, policy.
- The harness learns from scored traces, checks, and route outcomes.
- Local actions are permissioned before high-risk tools run.
- The pixel Octopus changes color from real runtime state.
- Rust keeps the kernel fast and stable; tentacles can evolve around it.

## What Works Now

Octopus is still pre-`0.1`, but the local MVP already runs:

- clean-brain chat, goal refinement, Need exploration, and reviewable Need Queue
- seed tentacles for SWE work, computer-use diagnostics, repo maintenance, harness repair, bash-only execution, and structured JSON Feed
- OpenAI-compatible provider setup for chat, tentacle planning, and harness evolution
- local memory, route learning, Feed traces, feedback scoring, and three-heart heartbeat
- native HTML app for bootstrap, provider setup, exploration, context, traces, checks, grants, pet state, and harness review
- reviewable harness evolution artifacts instead of silent self-patching

## Install & Start

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus
octopus --version
octopus bootstrap
octopus doctor
octopus report
```

Open the local app:

```bash
octopus bridge
```

Then visit `http://127.0.0.1:8765/app.html` for bootstrap, provider setup, exploration, Need Queue, context, traces, checks, grants, and the pixel pet.

Use an LLM provider:

```bash
octopus provider save openai
# set OPENAI_API_KEY, then:
source .octopus/llm.env
octopus provider check
```

Try the core loop:

```bash
octopus goal set "make this repo easier to use"
octopus explore "what should the brain ask next?"
octopus explore --save "what should the brain ask next?"
octopus needs
octopus needs take 1
octopus context observe .
octopus think swe-agent observe README.md
octopus need observe README.md
octopus traces
octopus feedback 1 satisfied "useful feed"
octopus beat 200
octopus pet
```

## Built-In Tentacles

- `swe-agent`: read, edit, patch, inspect, test
- `computer-use-agent`: browser/window diagnostics, clipboard, MCP, local desktop probes
- `repo-maintainer`: repo inspection, PR drafts, grant-bound publishing
- `harness-repair-agent`: diagnoses state, traces, checks, evolution artifacts, and adapters
- `bash-only`: transparent write-and-run execution
- `json-feed`: Python `octopus-json-v1` runtime seed
- `visual`: pixel Octopus state layer

## Links

- Product app: [docs/app.html](docs/app.html)
- Pixel pet: [docs/pet.html](docs/pet.html)
- Architecture: [docs/architecture.md](docs/architecture.md)
- Quick walkthrough: [docs/quickstart.md](docs/quickstart.md)
- Tentacle contract: [tentacles/README.md](tentacles/README.md)
