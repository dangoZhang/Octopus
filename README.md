# Octopus 🐙

Keep the main agent focused on the goal. Let local tentacles handle the tool work.

[Install](#quick-install--use) · [Local App](https://dangozhang.github.io/Octopus/app.html) · [Docs](https://dangozhang.github.io/Octopus/docs.html) · [中文](README.zh-CN.md)

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

**Fields are a peer pool.**

The `v0.2.0` target is broad field adaptation across math, search, code, SWE, research, computer-use, IB work, and robotics. Octopus keeps those fields in one Goal pool. `--workers n` only changes how many execution slots are open; it does not turn the fields into a queue.

Post-`v0.2.0`, `0.2.x` releases pre-evolve more installable field packs toward `v0.3.0`. The first expansion packs are writing and translation.

**The Octopus shows state.**

The pixel pet is a read-only desktop observer. It watches `.octopus/state.json`: head color marks Need, moving tentacles mark action, bubbles mark code-as-harness work, blue marks evolution, red marks blocked, and green marks Feed.

## Quick Install & Use

Current status: `v0.2.0`. The old `v0.1.0` release artifacts were removed; this is the first usable public line with desktop pet observation and real harness-evolution evidence.

```bash
curl -fsSL https://dangozhang.github.io/Octopus/install.sh | sh
octopus --version
```

Then run one local Goal/Need/Feed loop and open the app:

```bash
octopus first-run "make this repo easier to use"
octopus start --open
```

You should see `.octopus/state.json`, one Feed summary, and the local Goal/Need/Feed app at `http://127.0.0.1:8765/app.html`. `start --open` keeps the app server running; use `start --check` when you only need evidence.

Direct GitHub install:

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus first-run "make this repo easier to use"
octopus start --open
```

Live models are optional. Octopus can use Codex login, API-key providers, local OpenAI-compatible servers, or gateway routers, but provider setup is runtime plumbing. The user path stays Goal-first. See [Quickstart](docs/quickstart.md) for model setup.

## License

MIT.
