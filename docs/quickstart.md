# Quick Install & Use

## Source Run

```bash
cargo test
tmp=$(mktemp -d)
repo=$PWD
cargo run -q -p octopus-core -- demo dangoZhang/Octopus
cargo run -q -p octopus-core -- catalog
cargo run -q -p octopus-core -- manifests
cargo run -q -p octopus-core -- --lang zh env
cargo run -q -p octopus-core -- --state "$tmp/state.json" adapt
cargo run -q -p octopus-core -- --state "$tmp/state.json" install research
cargo run -q -p octopus-core -- --state "$tmp/state.json" install swe-agent
cargo run -q -p octopus-core -- --state "$tmp/state.json" installed
cargo run -q -p octopus-core -- --state "$tmp/state.json" chat "build a clean-brain agent"
cargo run -q -p octopus-core -- --state "$tmp/state.json" need observe .
cargo run -q -p octopus-core -- --state "$tmp/state.json" doctor
cargo run -q -p octopus-core -- --state "$tmp/state.json" chat "make tools think"
cargo run -q -p octopus-core -- --state "$tmp/state.json" beat 200
cargo run -q -p octopus-core -- --state "$tmp/state.json" goal
```

## Tentacle Run

```bash
(cd "$tmp" && cargo run -q --manifest-path "$repo/Cargo.toml" -p octopus-core -- scaffold my-feed python)
(cd "$tmp" && cargo run -q --manifest-path "$repo/Cargo.toml" -p octopus-core -- probe my-feed observe README.md)
(cd "$tmp" && cargo run -q --manifest-path "$repo/Cargo.toml" -p octopus-core -- scaffold native-feed rust)
(cd "$tmp" && cargo run -q --manifest-path "$repo/Cargo.toml" -p octopus-core -- manifests tentacles)
```

Known runtimes get starter code. Any other runtime gets a manifest and `tools/feed` contract so the tentacle can add its own executable adapter.

## Optional LLM

```bash
OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" \
  cargo run -q -p octopus-core -- llm "provider check"

OCTOPUS_LLM_MANIFEST=1 OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" \
  cargo run -q -p octopus-core -- --state "$tmp/state.json" --json need observe README.md 1 1
```

## Self-Iteration

```bash
cargo run -q -p octopus-core -- --state "$tmp/state.json" self-iterate dangoZhang/Octopus
cargo run -q -p octopus-core -- --state "$tmp/state.json" oauth github dangoZhang/Octopus
cargo run -q -p octopus-core -- --state "$tmp/state.json" self-iterate dangoZhang/Octopus
cargo run -q -p octopus-core -- evolve swe-agent "improve repository observation feed quality"
tentacles/repo-maintainer/tools/patch_queue.sh "$tmp" dangoZhang/Octopus "build a clean-brain agent"
```

## Expected Signals

```text
Octopus demo
installed swe-agent
== project ==
Octopus doctor
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
status: Satisfied
```
