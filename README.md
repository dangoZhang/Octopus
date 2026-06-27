# Octopus 🐙

> A clean brain for agents, with tools that can think, act, and improve outside the main model context.

Most agents make one model carry everything: task, memory, tool schemas, shell details, logs, routing, and repair notes. Octopus splits that burden. The main brain asks for the cognitive supply it needs; each tentacle owns its prompt, tool metadata, runtime code, permissions, and improvement trail, then sends back compact evidence.

That small shift keeps reasoning clean while the harness keeps learning from real Feed, checks, grants, and scores.

## Think Less About Tools

- 🐙 **Clean brain**: the model sees only `Goal + Mem + Need + Feed`.
- 🧠 **Thinking tentacles**: SWE, computer-use, repo-maintainer, harness-repair, bash-only, and JSON Feed seeds carry their own tool-side planning.
- 🫀 **Three beats**: heartbeat keeps liveness, memory beat compacts recall, harness beat turns failed checks or Feed traces into reviewable repair plans.
- 🎨 **Color state**: the pixel Octopus changes color from real Need/Feed/beat events, with square fallback for chat shells.
- 🔐 **Local grants**: risky tool actions require explicit scopes before runtime execution.
- ⚙️ **Rust kernel**: the non-evolvable base stays typed, fast, portable, and hard to accidentally rewrite.

## Install & Run

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus
octopus --version
octopus bootstrap
octopus bridge
```

Open `http://127.0.0.1:8765/app.html`.

Try the core loop:

```bash
octopus goal set "make this repo easier to use"
octopus brain "what should the brain ask next?"
octopus brain --session "what should the brain ask next?"
octopus brain --apply-json '{"summary":"external chat explored","needs":[{"kind":"verify","query":"goal evidence stays clean"}]}' --save "what should the brain ask next?"
octopus explore --save "what should the brain ask next?"
octopus needs
octopus context observe .
octopus need observe README.md
octopus repair .
octopus feedback 1 satisfied "useful evidence"
octopus beat 200
octopus pet
```

Add a live OpenAI-compatible provider when you want model-backed brain or tentacle planning:

```bash
octopus provider save openai
source .octopus/llm.env
octopus provider check
octopus brain --live --save "what should the brain ask next?"
octopus brain --session --live "what should the brain ask next?"
```

## Use It Today

- Bootstrap a local harness and seed tentacles in one command.
- Inspect the exact clean-brain context before a model call.
- Write external Chat session files with prompt, messages, reply template, optional live draft, and apply command.
- Import external chat replies into Goal or Need Queue without Feed execution.
- Let tentacles plan from their own manifest, tools, grants, and optional LLM.
- Run structured Feed, store traces, score outcomes, and update routing.
- Generate reviewable harness repair sessions, optional repair drafts, and evolution apply plans.
- Use the native HTML app for providers, external brain replies, Need Queue, repairs, Feed tests, grants, traces, preflight, and the pixel Octopus.
- Gate releases with `octopus preflight`, a generated script, and a real-machine record.

## Proof

```text
Octopus preflight
target: 0.1.0
version: 0.0.8
brain: Goal + Mem + Need + Feed
tentacle: Need + Tool + Action + Tool + Action -> Feed
next: octopus preflight record, octopus preflight script, octopus report
```

Status: pre-`0.1`; `0.1.0` is reserved for the first release-ready build with recorded real-machine testing.

## Links

- App: [docs/app.html](docs/app.html)
- Pixel Octopus: [docs/pet.html](docs/pet.html)
- Install guide: [docs/quickstart.md](docs/quickstart.md)
- Architecture: [docs/architecture.md](docs/architecture.md)
- Real-machine gate: [docs/real-machine-test.md](docs/real-machine-test.md)
- Tentacle contract: [tentacles/README.md](tentacles/README.md)
- License: MIT
