# Field Packs

Field Packs are the v0.2.0 harness layer for adapting Octopus to different task fields without hardcoding one workflow per domain.

Each pack describes the task shape, required capabilities, permission boundary, verifier, trajectory labels, and one or more mini tasks. A tentacle can then choose or generate an implementation while the core keeps the Need -> Feed contract stable.

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
```

## Pack Rules

- Describe what the task needs, not how a tool must implement it.
- Keep permissions explicit. Observation, local writes, network use, external accounts, and physical actions must be separate.
- Verifiers should produce pass, fail, or partial with an error category.
- Mini tasks must be small enough for repeated evolution runs.
- Trajectory labels should make failed runs reusable.

## Human Role

Octopus should own normal tentacle iteration from trajectories. Humans should improve templates, verifiers, permission boundaries, and self-repair infrastructure when Octopus cannot recover.
