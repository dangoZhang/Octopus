#!/usr/bin/env bash
set -euo pipefail

root="${1:?usage: draft_pr.sh <root> <repo> <objective...>}"
repo="${2:?usage: draft_pr.sh <root> <repo> <objective...>}"
shift 2
objective="${*:-improve Octopus usability}"

cd "$root"
out_dir=".octopus/self-iteration"
mkdir -p "$out_dir"

slug=$(printf '%s' "$objective" \
  | tr '[:upper:]' '[:lower:]' \
  | sed 's/[^a-z0-9][^a-z0-9]*/-/g; s/^-//; s/-$//' \
  | cut -c 1-48)
if [ -z "$slug" ]; then
  slug="update"
fi

branch="octopus/$slug"
branch_plan="$out_dir/BRANCH_PLAN.md"
pr_draft="$out_dir/PR_DRAFT.md"
title=$(printf '%s' "$objective" | awk '{print toupper(substr($0,1,1)) substr($0,2)}')

cat > "$branch_plan" <<EOF_PLAN
# Branch Plan

Repository: $repo
Branch: $branch
Objective: $objective

Steps:
- inspect repository health
- compare docs, manifests, and tests
- prepare the smallest useful patch
- run checks before publishing
- open a pull request with compact evidence

Guardrails:
- never push to main directly
- prefer small pull requests
- keep the kernel small enough to audit
EOF_PLAN

cat > "$pr_draft" <<EOF_PR
# Draft PR

Title: $title
Branch: $branch

## Summary
- $objective

## Checks
- [ ] cargo test
- [ ] cargo clippy --all-targets -- -D warnings
- [ ] PYTHONPATH=src python -m unittest discover -s tests -q

## Guardrails
- never push to main directly
- prefer small pull requests
- keep the kernel small enough to audit
EOF_PR

echo "branch=$branch"
echo "branch_plan=$branch_plan"
echo "pr_draft=$pr_draft"
