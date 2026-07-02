
Legacy `*.pyfrag` files preserve earlier field trajectories. New field work should use `../workers/<field>/<mini-task>/main.go` plus reusable helpers under `../internal/fieldworker/`.

Each legacy fragment owns one narrow `field_pack` plus `field_mini_task` implementation and starts with its own `if field == ...` guard. `tools/check_repair_templates.py` still compiles and executes these templates so old Feed evidence stays reviewable. A template may return `satisfied` only with live or artifact-backed evidence; when evidence is missing it should return `partial` with gap metadata.
