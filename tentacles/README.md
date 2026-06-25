# Tentacles

Each tentacle is code-as-harness: LLM brain prompt, tool metadata, code implementation, and evolution policy.

Inspect installable manifests:

```bash
cargo run -q -p octopus-core -- manifests
```

- `swe-agent`: repo work through `read`, `edit`, inspection, patch, and test tools.
- `repo-maintainer`: OAuth-bounded self-iteration with repo inspection and PR draft data.
- `computer-use-agent`: local UI work through `mcp`, `bash`, screenshots, URLs, and desktop probes.
- `bash-only`: every action becomes a readable script before it runs.
- `visual`: color-changing status pet for heartbeat, memory, harness, blocked, and success states.

Shell is only one seed runtime. A tentacle can evolve into Python, TypeScript, Rust, MCP, HTTP, native tools, or mixed runtimes while the kernel contract stays stable.

Use `contract: octopus-json-v1` when a tool wants the full Need/tool/tentacle JSON envelope on stdin. Omit it for legacy executable entrypoints.
