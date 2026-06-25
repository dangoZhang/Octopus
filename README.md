# Octopus

> Independent-thinking tools, clean agent brain.

## Insight

Most agents make the brain carry tools, memory, skills, and execution flow. Octopus keeps the brain clean: it only says what cognition it needs.

Tentacles think during execution. A tentacle is an LLM brain prompt, tool metadata, code implementation, and evolution policy. The harness learns which feed works from data.

Initial tentacles are editable code-as-harness: SWE (`read/edit`), computer-use (`mcp/bash`), and bash-only as one transparent seed runtime.

Shell is only a seed runtime. A tentacle can declare `contract: octopus-json-v1` and run Python, Node, MCP, HTTP, native code, or any custom adapter.

Three hearts keep it alive: heartbeat, memory evolution, and harness route evolution. Color change is a visual pet/status layer outside the kernel.

The mechanism is `Need -> Feed -> Feedback`. The outcome is less tool burden and stronger tools.

## Quick Install & Use

```bash
cargo test
tmp=$(mktemp -d)
repo=$PWD
cargo run -q -p octopus-core -- catalog
cargo run -q -p octopus-core -- manifests
cargo run -q -p octopus-core -- --lang zh env
cargo run -q -p octopus-core -- --state "$tmp/state.json" adapt
cargo run -q -p octopus-core -- --state "$tmp/state.json" install research
cargo run -q -p octopus-core -- --state "$tmp/state.json" install swe-agent
cargo run -q -p octopus-core -- --state "$tmp/state.json" installed
cargo run -q -p octopus-core -- --state "$tmp/state.json" chat "build a clean-brain agent"
cargo run -q -p octopus-core -- --state "$tmp/state.json" need observe .
cargo run -q -p octopus-core -- --state "$tmp/state.json" status
cargo run -q -p octopus-core -- --state "$tmp/state.json" chat "make tools think through tentacles"
cargo run -q -p octopus-core -- --state "$tmp/state.json" beat 200
cargo run -q -p octopus-core -- --state "$tmp/state.json" goal
cargo run -q -p octopus-core -- --state "$tmp/state.json" self-iterate dangoZhang/Octopus
cargo run -q -p octopus-core -- --state "$tmp/state.json" oauth github dangoZhang/Octopus
cargo run -q -p octopus-core -- --state "$tmp/state.json" self-iterate dangoZhang/Octopus
OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" cargo run -q -p octopus-core -- llm "provider check"
OCTOPUS_LLM_MANIFEST=1 OCTOPUS_LLM_MODEL=gpt-4.1-mini OCTOPUS_LLM_API_KEY="$OPENAI_API_KEY" cargo run -q -p octopus-core -- --state "$tmp/state.json" --json need observe README.md 1 1
(cd "$tmp" && cargo run -q --manifest-path "$repo/Cargo.toml" -p octopus-core -- scaffold my-feed python)
cargo run -q -p octopus-core -- evolve swe-agent "improve repository observation feed quality"
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
adapted tentacles: visual, repo-maintainer, swe-agent, bash-only, computer-use-agent
installed research
installed swe-agent (runtimes: shell)
profiles: memory, visual, repo-maintainer, swe-agent, code, research, bash-only, computer-use-agent
tentacles: visual(static-html), repo-maintainer(shell), swe-agent(shell), bash-only(shell), computer-use-agent(mcp,shell)
goal: build a clean-brain agent
== project ==
Octopus status
hearts: heartbeat=ready, memory=1 memories, harness=2 routes
warnings: no active OAuth grants
next: cargo run -q -p octopus-core -- --state /tmp/octopus/state.json beat 200
turn 2: remembered m2
refinements: 1
heartbeat: alive
memory: compacted 0 memories
harness: evolved 2 routes
mode: report-only
oauth grant active: github:dangoZhang/Octopus
mode: pr-ready
draft branch: octopus/build-a-clean-brain-agent
provider check
"plan_source": "llm"
manifest: /path/to/tentacles/my-feed/manifest.json
proposal: .octopus/evolution/swe-agent/PROPOSAL.md
queue=.octopus/self-iteration/PATCH_QUEUE.md
已记住 m3
工具不进大脑
verified: the brain does not name tools
plan: selected first matching tool
```
