# Tentacles

Each tentacle is code-as-harness: a small manifest plus editable tools.

- `swe-agent`: `read`, `edit`, repo inspection, patch writing, test running.
- `computer-use-agent`: `mcp`, `bash`, screenshot, open-url, desktop helpers.
- `bash-only`: writes every tool call to a `.sh` file and runs it.

The Rust kernel stays clean. These files are intentionally ordinary code so Octopus can later edit, test, and improve them through a repo tentacle.
