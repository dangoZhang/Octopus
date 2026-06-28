# Octopus 🐙

Keep the main agent focused on the goal. Let local tentacles handle the tool work.

[Install](#quick-install--use) · [Product Demo](https://dangozhang.github.io/Octopus/demo.html) · [Docs](https://dangozhang.github.io/Octopus/docs.html) · [Try App](https://dangozhang.github.io/Octopus/app.html?demo=hello) · [中文](README.zh-CN.md)

Biological octopuses distribute control through the body. The central brain sets direction; the arms do much of the sensing and adjustment close to the world.

Octopus brings that shape to agents. The main model keeps the goal context small. Tentacles stay near the tools, handle the noisy steps, and return a short result the brain can use.

That is the product idea: a cleaner brain, smarter tools, and a harness that can improve without turning the main context into a tool log.

```text
Goal -> Brain -> Need -> Tentacle -> Tool work -> Feed -> Brain
Heartbeat -> run data -> memory and harness updates
```

## Why Octopus

**Tools become local nervous systems.**

Most agents treat tools as passive calls. Octopus treats each tool workflow as a small local worker: it can inspect the environment, choose the next step, check the result, and return compact Feed.

**Need stays separate from implementation.**

The brain asks for what it needs from the world. It does not need to carry shell syntax, browser choreography, repo commands, or provider plumbing.

This separation is first-class in the runtime: Goal, Need, Feed, and tentacle execution are different surfaces, not one prompt convention.

**Harness changes without dirtying the brain.**

Seed tentacles live under `tentacles/`. Their prompts, manifests, tools, and repair policy can be inspected and changed while the core Goal -> Need -> Feed loop stays stable.

**The Octopus shows state.**

The pixel pet is not decoration. It is the smallest visible surface for waiting, running, memory, harness, blocked, and success states.

## Quick Install & Use

```bash
curl -fsSL https://dangozhang.github.io/Octopus/install.sh | sh
octopus --version
octopus start --open
```

Then run one local loop:

```bash
octopus first-run "make this repo easier to use"
octopus chat "prefer one small evidence-backed improvement"
octopus pet
```

You should see the local app at `http://127.0.0.1:8765/app.html`, a `.octopus/state.json` file, one Feed summary, and a pixel Octopus state.

Direct GitHub install:

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus start --check
octopus start --open
```

## Models

Codex login:

```bash
octopus provider save codex
source .octopus/llm.env
octopus provider check
octopus first-run --live "make this repo easier to use"
```

API-key provider:

```bash
octopus provider save openai
source .octopus/llm.env
export OPENAI_API_KEY=...
octopus provider check
```

Local OpenAI-compatible servers and gateway routers use the same provider shape.

## Current Shape

Octopus already starts as one local product: app, pet, docs, recipes, installer page, provider checks, seed tentacles, and release evidence.

The release evidence is recorded in [docs/real-machine-test.md](docs/real-machine-test.md). The v0.1.0 line includes installed-binary checks, local app checks, provider matrix evidence, and minimal SWE/Claw/Wild benchmark records.

The GitHub Pages app is a zero-install taste of the idea. It sends requests directly to the endpoint you enter; the project does not proxy API keys.

## Docs

- [Product Demo](docs/demo.html)
- [Docs Tutorial](docs/docs.html)
- [5-minute Use Guide](docs/use.html)
- [Recipes](docs/recipes.html)
- [Architecture](docs/architecture.md)
- [Research Map](docs/references.md)
- [v0.2.0 Field Adaptation Harness](docs/field-adaptation.md)
- [Current Gap Log](docs/product-gap.md)

## Road to v0.2.0

v0.2.0 focuses on field adaptation. Octopus should learn how different task fields behave, then improve the right tentacle from trajectories, errors, and repairs.

## License

MIT.
