# Quick Install & Use

## Source Run

```bash
cargo install --git https://github.com/dangoZhang/Octopus --locked --package octopus-core --bin octopus --force
octopus --version
octopus doctor
```

## Source Build

```bash
cargo test
cargo install --path crates/octopus-core --bin octopus --force
tmp=$(mktemp -d)
octopus demo dangoZhang/Octopus
octopus catalog
octopus skills
octopus manifests
octopus --lang zh env
octopus --state "$tmp/state.json" init
octopus --state "$tmp/state.json" install research
octopus --state "$tmp/state.json" install json-feed
octopus --state "$tmp/state.json" install swe-agent
octopus --state "$tmp/state.json" install computer-use-agent
octopus --state "$tmp/state.json" install bash-only
octopus --state "$tmp/state.json" installed
octopus --state "$tmp/state.json" chat "build a clean-brain agent"
octopus --state "$tmp/state.json" need observe .
octopus --state "$tmp/state.json" pet
octopus --state "$tmp/state.json" doctor
octopus --state "$tmp/state.json" chat "make tools think"
octopus --state "$tmp/state.json" beat 200
octopus --state "$tmp/state.json" goal
```

## Tentacle Run

```bash
(cd "$tmp" && octopus scaffold my-feed python)
(cd "$tmp" && octopus probe my-feed observe README.md)
(cd "$tmp" && octopus scaffold native-feed rust)
(cd "$tmp" && octopus manifests tentacles)
```

Known runtimes get starter code. Any other runtime gets a manifest and `tools/feed` contract so the tentacle can add its own adapter, MCP bridge, native binary, or remote executor.

`json-feed` is the built-in non-shell seed: Python receives the full `octopus-json-v1` envelope and returns structured feedback.

## Optional LLM

```bash
OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" \
  octopus llm "provider check"

OCTOPUS_CHAT_LLM=1 OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" \
  octopus --state "$tmp/state.json" chat "make the harness self-evolve"

OCTOPUS_LLM_MANIFEST=1 OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" \
  octopus --state "$tmp/state.json" --json need observe README.md 1 1

OCTOPUS_LLM_EVOLVE=1 OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" \
  octopus --state "$tmp/state.json" evolve swe-agent "improve observe feed"

OCTOPUS_CHAT_LLM_PREFIX=OCTOPUS_LLM OCTOPUS_MANIFEST_LLM_PREFIX=OCTOPUS_LLM octopus doctor
```

## Self-Iteration

```bash
octopus --state "$tmp/state.json" self-iterate dangoZhang/Octopus
octopus --state "$tmp/state.json" oauth github dangoZhang/Octopus
octopus --state "$tmp/state.json" self-iterate dangoZhang/Octopus
octopus evolve swe-agent "improve repository observation feed quality"
octopus evolve apply swe-agent runtime_code
octopus evolve score swe-agent 03-runtime-code satisfied "patch improved feed"
tentacles/repo-maintainer/tools/patch_queue.sh "$tmp" dangoZhang/Octopus "build a clean-brain agent"
```

`evolve` writes `PROPOSAL.md`, `PATCH_CANDIDATES.md`, `PATCH_DRAFTS.md`, `patches/`, `apply/`, optional `.patch` files, and `proposal.json` under `.octopus/evolution/<tentacle>/`.

## Expected Signals

```text
Octopus demo
Octopus init
installed swe-agent
json-feed observe
Octopus doctor
pixel: 🟥
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
status: Satisfied
```
