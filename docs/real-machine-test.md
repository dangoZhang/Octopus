# Real-Machine Test Gate

Required before `0.1.0` and every later version tag.

## Machine

- Date:
- Tester:
- Machine:
- OS:
- Shell:
- Rust:
- Python:
- Git commit:
- Version tag:

## Install And Update

```bash
cargo install --git https://github.com/dangoZhang/Octopus octopus-core --locked --bin octopus --force
octopus --version
octopus doctor
```

Result:

## Core Loop

```bash
tmp=$(mktemp -d)
octopus --state "$tmp/state.json" init
octopus --state "$tmp/state.json" skills
octopus --state "$tmp/state.json" install swe-agent
octopus --state "$tmp/state.json" install computer-use-agent
octopus --state "$tmp/state.json" install bash-only
octopus --state "$tmp/state.json" chat "build a clean-brain agent"
octopus --state "$tmp/state.json" need observe .
octopus --state "$tmp/state.json" pet harness
octopus --state "$tmp/state.json" beat 200
octopus --state "$tmp/state.json" doctor
```

Result:

## Harness Evolution

```bash
octopus --state "$tmp/state.json" evolve swe-agent "improve observe feed"
octopus --state "$tmp/state.json" evolve score swe-agent 03-runtime-code satisfied "real-machine check"
```

Result:

## HTML UI

- Open `docs/app.html`.
- Open `docs/pet.html?state=harness`.
- Confirm pixel body color changes and chat fallback shows `🟥 🐙`.
- Run `octopus pet` after `beat` and confirm `event: harness beat` or matching latest action event appears.

Result:

## Known Issues

- 

## Tag Decision

- Pass or fail:
- Follow-up:
