# Octopus

> Independent-thinking tools, clean agent brain.

## Insight

Most agents make the brain carry tools, memory, skills, and execution flow. Octopus keeps the brain clean: it only says what cognition it needs.

Tentacles think during execution. A tentacle is an LLM brain prompt, tool metadata, code implementation, and evolution policy. The harness learns which feed works from data.

Initial tentacles are editable code-as-harness: SWE (`read/edit`), computer-use (`mcp/bash`), and bash-only as one transparent seed runtime.

Three hearts keep it alive: heartbeat, memory evolution, and harness route evolution. Color change is a visual pet/status layer outside the kernel.

The mechanism is `Need -> Feed -> Feedback`. The outcome is less tool burden and stronger tools.

## Quick Install & Use

```bash
cargo test
tmp=$(mktemp -d)
cargo run -q -p octopus-core -- catalog
cargo run -q -p octopus-core -- manifests
cargo run -q -p octopus-core -- --lang zh env
cargo run -q -p octopus-core -- --state "$tmp/state.json" adapt
cargo run -q -p octopus-core -- --state "$tmp/state.json" install research
cargo run -q -p octopus-core -- --state "$tmp/state.json" install swe-agent
cargo run -q -p octopus-core -- --state "$tmp/state.json" installed
cargo run -q -p octopus-core -- --state "$tmp/state.json" need observe .
cargo run -q -p octopus-core -- --state "$tmp/state.json" chat "build a clean-brain agent"
cargo run -q -p octopus-core -- --state "$tmp/state.json" chat "make tools think through tentacles"
cargo run -q -p octopus-core -- --state "$tmp/state.json" beat 200
cargo run -q -p octopus-core -- --state "$tmp/state.json" goal
cargo run -q -p octopus-core -- --state "$tmp/state.json" self-iterate dangoZhang/Octopus
cargo run -q -p octopus-core -- --state "$tmp/state.json" oauth github dangoZhang/Octopus
cargo run -q -p octopus-core -- --state "$tmp/state.json" self-iterate dangoZhang/Octopus
tentacles/repo-maintainer/tools/patch_queue.sh "$tmp" dangoZhang/Octopus "build a clean-brain agent"
cargo run -q -p octopus-core -- --state "$tmp/state.json" --lang zh need remember "工具不进大脑"
cargo run -q -p octopus-core -- --state "$tmp/state.json" need recall 工具
cargo run -q -p octopus-core --example thinking_tentacle
```

Output includes:

```text
Installable tentacles: research, code, memory, visual, repo-maintainer, swe-agent, computer-use-agent, bash-only
Tentacle manifests:
- computer-use-agent: brain=llm, runtime=mcp,shell, tools=5, status=ok
- visual: brain=llm, runtime=static-html, tools=1, status=ok
推荐触手: memory, visual, repo-maintainer, swe-agent, code, research, bash-only, computer-use-agent
adapted tentacles:
installed research
installed swe-agent (runtimes: shell)
profiles:
tentacles:
== project ==
goal: build a clean-brain agent
turn 2: remembered m2
refinements: 1
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
mode: report-only
oauth grant active: github:dangoZhang/Octopus
mode: pr-ready
draft branch: octopus/build-a-clean-brain-agent
queue=.octopus/self-iteration/PATCH_QUEUE.md
已记住 m3
工具不进大脑
verified: the brain does not name tools
plan: selected first matching tool
```
