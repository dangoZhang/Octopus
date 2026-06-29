# Field Packs

Field Packs are the `0.1.x` foundation for the `v0.2.0` field-adaptation target. They let Octopus understand different task fields without hardcoding one workflow per domain.

Each pack describes the task shape, required capabilities, permission boundary, verifier, trajectory labels, and one or more mini tasks. A tentacle can then choose or generate an implementation while the core keeps the Need -> Feed contract stable.

Octopus core loads these packs from `field-packs/` or the embedded release copy. The selected field is written into Need context and Feed traces as `field_pack`, so later harness evolution can group failures by domain.

The packs describe task shape. Concrete tentacle behavior should be iterated by Octopus from trajectories, verifier results, and repair attempts.

Current `0.1.x` scope: keep the required v0.2 field goals as peer slots in one pool, allow expansion packs such as writing, open worker slots from that pool, record traces, record verifier results, and route failed field evidence into harness repair. The field list is not a backlog. Worker count controls concurrency only. Required seed packs now have three mini tasks, and all three layers are satisfied after repair/rerun/score cycles.

## Contract

```text
Goal -> field signal -> Need -> tentacle plan
     -> action trajectory -> Feed -> verifier result
     -> harness repair/evolution -> reuse
```

## Files

```text
field-packs/
  index.json
  field-pack.schema.json
  _template/field-pack.json
  math/field-pack.json
  search/field-pack.json
  code/field-pack.json
  swe/field-pack.json
  research/field-pack.json
  computer-use/field-pack.json
  ib/field-pack.json
  robotics/field-pack.json
  write/field-pack.json
```

## Pack Rules

- Describe what the task needs, not how a tool must implement it.
- Keep `aliases` in the pack data; they are the editable names users and benchmarks use for the field.
- Keep permissions explicit. Observation, local writes, network use, external accounts, and physical actions must be separate.
- Verifiers should produce pass, fail, or partial with an error category.
- Mini tasks must be small enough for repeated evolution runs.
- Put easier tasks before harder tasks inside one field. Math, search, code, SWE, research, computer-use, IB, and robotics stay peer slots in one worker pool.
- Treat `--workers n` as execution capacity. `n=1` opens one visible slot from the same parallel pool; larger values open more slots without changing the field objective set.
- Trajectory labels should make failed runs reusable.

## Human Role

Octopus should own normal tentacle iteration from trajectories. Humans should improve templates, verifiers, permission boundaries, and self-repair infrastructure when Octopus cannot recover.

## Inspect

```sh
octopus fields
octopus fields summary
octopus fields match verify "dedupe search results and keep citations"
octopus fields score latest failed missing_source "citation coverage failed"
octopus evolve parallel --workers 4 --open "advance the peer field objectives toward v0.2.0"
octopus traces 10
```
