# Structure

Updated: 2026-06-28

## Tree

```text
Octopus/
├── README.md
├── README.zh-CN.md
├── Cargo.toml
├── structure.md
├── crates/octopus-core/
│   └── src/
│       ├── lib.rs             Stable kernel contracts, state, routing, providers, Feed traces.
│       ├── main.rs            CLI dispatch and product/backend aggregation.
│       ├── app_bridge.rs      Local app server, bridge policy, static app fallback, streaming.
│       ├── core_boundary.rs   Stable Rust vs editable harness boundary report.
│       └── release_gate.rs    Preflight records, benchmark evidence, real-machine records, release scripts.
├── tentacles/
│   ├── profile-registry/      Editable seed profile data.
│   ├── swe-agent/             Repo read/edit/patch/test tool-combo tentacle.
│   ├── computer-use-agent/    Desktop/browser/MCP/shell/clipboard tentacle.
│   ├── repo-maintainer/       Repo health and PR/publish tentacle.
│   ├── harness-repair-agent/  Harness diagnosis and repair tentacle.
│   ├── bash-only/             Transparent write-and-run harness.
│   ├── json-feed/             `octopus-json-v1` runtime seed.
│   └── visual/                Pixel pet state layer.
└── docs/
    ├── app.html               One-page local product app plus browser-only Try App demo.
    ├── tutorial.html          Product tutorial from browser demo to whole-project local run.
    ├── recipes.html           Goal-first recipes for browser, local repo, model, and evidence paths.
    ├── pet.html               Pixel Octopus.
    ├── architecture.md
    ├── product-gap.md
    ├── version-plan.md
    ├── real-machine-test.md
    └── zh/
        ├── tutorial.html      Chinese product tutorial from browser demo to local run.
        └── recipes.html       Chinese Goal-first recipes.
```

## Boundary

- Clean brain context: `Goal + Mem + Need + Feed`.
- Tentacle context: `Need + Tool + Action + Tool + Action -> Feed`.
- Stable Rust owns the kernel, product bridge, release gates, benchmark evidence checks, and local app shell.
- Editable code-as-harness owns Feed supply: tentacle prompts, tool metadata, runtime code, checks, permissions, and evolution policy.
- The product app lets the user change only Goal; Need/Feed execution, repair, scoring, provider setup, install/check, and evolution stay inside the agent loop or developer harness.

## Current Product Path

1. Try `docs/tutorial.html`, `docs/recipes.html`, and `docs/app.html` in the browser or install and run `octopus start --open`.
2. Enter one Goal or use the browser Try App buttons.
3. Octopus turns the Goal into Need, routes Feed through tentacles, records feedback, and pulses the three hearts.
4. The app shows pet state, current Need, current Feed, Output, and a browser-only API-key demo when no local bridge is present.
5. `benchmark record/check` collects the smallest SWE/Claw/Wild pass evidence.
6. `preflight`, provider matrix, real-machine records, and benchmark records remain release evidence before `0.1.0`.
