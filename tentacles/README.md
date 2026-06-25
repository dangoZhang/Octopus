# Tentacles

Each tentacle is code-as-harness: LLM brain prompt, tool metadata, code implementation, and evolution policy.

The core treats runtime as metadata plus an entrypoint contract. `.sh` files are current seed implementations, not the tentacle model.

Inspect installable manifests:

```bash
octopus manifests
tmp=$(mktemp -d)
(cd "$tmp" && octopus scaffold my-feed python)
(cd "$tmp" && octopus probe my-feed observe README.md)
(cd "$tmp" && octopus scaffold native-feed rust)
```

- `swe-agent`: repo work through `read`, `edit`, inspection, patch, and test tools.
- `repo-maintainer`: OAuth-bounded self-iteration with repo inspection and PR draft data.
- `computer-use-agent`: local UI work through `mcp`, `bash`, screenshots, URLs, and desktop probes.
- `bash-only`: every action becomes a readable script before it runs.
- `visual`: color-changing status pet for heartbeat, memory, harness, blocked, and success states.

Shell is only one seed runtime. A tentacle can evolve into Python, TypeScript, Rust, MCP, HTTP, native tools, or mixed runtimes while the kernel contract stays stable. Unknown runtimes scaffold as manifest-only and become installable after the tentacle adds its executable adapter.

Use `contract: octopus-json-v1` when a tool wants the full Need/tool/tentacle JSON envelope on stdin. Omit it for legacy executable entrypoints.
