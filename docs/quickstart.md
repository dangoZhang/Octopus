# Quick Install & Use

## Source Run

```bash
cargo test
cargo install --path crates/octopus-core --bin octopus --force
tmp=$(mktemp -d)
octopus demo dangoZhang/Octopus
octopus catalog
octopus manifests
octopus --lang zh env
octopus --state "$tmp/state.json" adapt
octopus --state "$tmp/state.json" install research
octopus --state "$tmp/state.json" install swe-agent
octopus --state "$tmp/state.json" installed
octopus --state "$tmp/state.json" chat "build a clean-brain agent"
octopus --state "$tmp/state.json" need observe .
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

Known runtimes get starter code. Any other runtime gets a manifest and `tools/feed` contract so the tentacle can add its own executable adapter.

## Optional LLM

```bash
OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" \
  octopus llm "provider check"

OCTOPUS_LLM_MANIFEST=1 OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" \
  octopus --state "$tmp/state.json" --json need observe README.md 1 1
```

## Self-Iteration

```bash
octopus --state "$tmp/state.json" self-iterate dangoZhang/Octopus
octopus --state "$tmp/state.json" oauth github dangoZhang/Octopus
octopus --state "$tmp/state.json" self-iterate dangoZhang/Octopus
octopus evolve swe-agent "improve repository observation feed quality"
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
