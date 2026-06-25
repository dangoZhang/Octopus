# Self-Iteration Loop

Octopus should eventually improve its own repository, but only through an explicit authorization boundary.

## Contract

The main brain emits needs:

- inspect repo health
- compare docs and code
- verify tests
- propose an improvement
- remember prior outcomes

The repo tentacle does implementation work:

- reads files and issues
- edits code or docs
- runs CI locally
- opens a pull request
- reports compact feedback

## OAuth Boundary

LLM OAuth is a capability grant, not a hidden background permission. Without it, Octopus can plan and report. With it, Octopus may create branches, commits, and PRs inside the allowed repository scope.

## Guardrails

- never push to `main` directly
- prefer small PRs
- run Rust and Python tests before proposing merge
- store route outcomes in harness state
- keep the kernel small enough to audit

## Target Loop

```text
Need: improve Octopus usability
Feed: repo tentacle inspects code, docs, CI, issues, and papers
Feedback: patch, tests, PR link, route score
```

This keeps self-improvement outside the clean brain while letting the harness learn which project improvements work.

