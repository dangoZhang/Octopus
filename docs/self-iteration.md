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

```bash
tmp=$(mktemp -d)
octopus --state "$tmp/state.json" self-iterate dangoZhang/Octopus
octopus --state "$tmp/state.json" oauth github dangoZhang/Octopus
octopus --state "$tmp/state.json" self-iterate dangoZhang/Octopus
OCTOPUS_PR_DRY_RUN=1 octopus --state "$tmp/state.json" self-iterate pr dangoZhang/Octopus "improve usability"
```

The first plan is `report-only`. After the grant, the repo-maintainer plan becomes `pr-ready`. The PR command uses the grant boundary; remove `OCTOPUS_PR_DRY_RUN=1` only when the local branch and `gh` auth are ready.

The repo-maintainer tentacle can also write local artifacts:

```bash
tentacles/repo-maintainer/tools/inspect_repo.sh .
tentacles/repo-maintainer/tools/github_status.sh dangoZhang/Octopus
tmp=$(mktemp -d)
tentacles/repo-maintainer/tools/patch_queue.sh "$tmp" dangoZhang/Octopus "improve usability"
tentacles/repo-maintainer/tools/draft_pr.sh "$tmp" dangoZhang/Octopus "improve usability"
OCTOPUS_PR_DRY_RUN=1 tentacles/repo-maintainer/tools/publish_pr.sh "$tmp" dangoZhang/Octopus octopus/improve-usability "Improve usability" "$tmp/.octopus/self-iteration/PR_DRAFT.md"
```

GitHub status, patch queue, draft data, and publish reports land under `.octopus/self-iteration/` in the selected workspace.

Harness evolution drafts stay local and auditable:

```bash
octopus evolve swe-agent "improve repository observation feed quality"
octopus evolve apply swe-agent runtime_code
octopus oauth octopus evolve:swe-agent harness:write
octopus evolve apply swe-agent runtime_code
octopus evolve score swe-agent 03-runtime-code satisfied "runtime patch improved feed"
octopus evolve recommend swe-agent "continue from scored feedback"
```

The draft lands under `.octopus/evolution/<tentacle>/` with `PROPOSAL.md`, `PATCH_CANDIDATES.md`, `PATCH_DRAFTS.md`, per-candidate files under `patches/`, and `proposal.json`. The proposal includes recent Feed traces and check history for that tentacle, so harness evolution can use real execution feedback. Runtime-code candidates point at the concrete traced or failing tool entrypoint when one is available. Patch candidates, drafts, and apply plans include a `feedback focus` block; LLM-generated candidates can also carry a provider-assisted unified diff for the declared target. `evolve recommend` uses scored outcomes plus matching check history to choose the next candidate and write an apply plan. `evolve apply` writes an apply plan under `apply/`; it stays in `needs_authorization` until the matching `octopus:evolve:<tentacle>` grant has `harness:write`, then writes a reviewable `.patch` file without applying it.

`beat 200` also feeds this loop. If recent check history has a failed or partial record for a known tentacle, the harness beat writes the proposal, recommendation, and apply plan automatically and prints the next grant or apply command. The native HTML app shows the same candidate, apply-plan preview, and next action in its Harness Beat panel.

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
