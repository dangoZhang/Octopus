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

The pixel pet is a read-only desktop observer. It watches `.octopus/state.json`: head color marks Need, moving tentacles mark action, bubbles mark code-as-harness work, blue marks evolution, red marks blocked, and green marks Feed.

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
octopus pet desktop
```

You should see the local app at `http://127.0.0.1:8765/app.html`, a `.octopus/state.json` file, one Feed summary, and one native desktop Octopus observing that state.

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

The release evidence is recorded in [docs/real-machine-test.md](docs/real-machine-test.md). The v0.1.0 line includes installed-binary checks, local app checks, provider matrix evidence, field-pool evidence, and minimal SWE/Claw/Wild benchmark records.

The GitHub Pages app is a zero-install taste of the idea. It sends requests directly to the endpoint you enter; the project does not proxy API keys.

## Docs

- [Product Demo](docs/demo.html)
- [Docs Tutorial](docs/docs.html)
- [5-minute Use Guide](docs/use.html)
- [Recipes](docs/recipes.html)
- [Architecture](docs/architecture.md)
- [Research Map](docs/references.md)
- [Field Adaptation TODO](docs/field-adaptation.md)
- [Current Gap Log](docs/product-gap.md)

## TODO

`0.1.x` is building the field-evolution base for `v0.2.0`. The eight fields form one parallel matrix; workers choose slots from that matrix.

Peer slots: `math`, `search`, `code`, `SWE`, `research`, `computer-use`, `IB`, and `robotics`.
Each slot uses the same Need -> Feed chain, verifier traces, repair templates, and harness evolution loop.

- [x] Native read-only desktop pet with Need bubbles, action bubbles, Goal view, evolution blue, and blocked red.
- [x] Feed traces record `field_pack`, verifier result, trajectory summary, and repair signal.
- [x] Field-specific repair code lives in editable harness templates outside the Rust kernel.
- [x] Failed field traces can produce reviewable harness patches, apply them with a grant, rerun, and score the result.
- [x] `mini-1/2/3` are training rungs inside each field. They do not create order across fields.

## License

MIT.
