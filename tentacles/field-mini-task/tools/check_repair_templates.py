#!/usr/bin/env python3
"""Verify every field-pack mini task has an editable repair template."""

from __future__ import annotations

import datetime as dt
import json
import re
import sys
import tempfile
from pathlib import Path


def repo_root(start: Path) -> Path | None:
    for candidate in [start, *start.parents]:
        if (candidate / "field-packs").is_dir() and (candidate / "tentacles").is_dir():
            return candidate
    return None


def tentacle_root(start: Path) -> Path:
    for candidate in [start, *start.parents]:
        if (candidate / "manifest.json").is_file() and (candidate / "repair-templates").is_dir():
            return candidate
    raise SystemExit("could not find field-mini-task root with manifest.json and repair-templates/")


def first_nonempty_line(path: Path) -> str:
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            return line.strip()
    return ""


def compact(value: object, limit: int = 240) -> str:
    text = " ".join(str(value or "").split())
    return text if len(text) <= limit else text[: limit - 3] + "..."


def rel(path: Path, root: Path) -> str:
    try:
        return str(path.relative_to(root))
    except ValueError:
        return str(path)


def has_artifact_backing(item: object, root: Path, session: Path) -> bool:
    if not isinstance(item, dict):
        return False
    candidates: list[object] = []
    for key in ("artifact", "artifact_path", "path", "source", "content"):
        candidates.append(item.get(key))
    metadata = item.get("metadata")
    if isinstance(metadata, dict):
        candidates.extend(metadata.values())
    for value in candidates:
        if not isinstance(value, str) or not value.strip():
            continue
        text = value.strip()
        if not looks_like_artifact_path(text):
            continue
        paths = [Path(text)]
        if not Path(text).is_absolute():
            paths.extend([root / text, session / text])
        if any(path_exists(path) for path in paths):
            return True
    return False


def looks_like_artifact_path(text: str) -> bool:
    if len(text) > 240 or "\n" in text or text[0] in "[{":
        return False
    path = Path(text)
    return (
        path.is_absolute()
        or text.startswith(".")
        or "/" in text
        or "\\" in text
        or bool(path.suffix)
    )


def path_exists(path: Path) -> bool:
    try:
        return path.exists()
    except OSError:
        return False


def seed_session(session: Path, field: str, task_id: str, expected_feed: str) -> None:
    session.mkdir(parents=True, exist_ok=True)
    (session / "TASK.json").write_text(
        json.dumps(
            {
                "schema_version": "octopus-field-mini-task-v1",
                "field": field,
                "mini_task": task_id,
                "need_kind": "verify",
                "need_query": f"Run {field} mini task {task_id}",
                "expected_feed": expected_feed,
                "context": {
                    "field_pack": field,
                    "field_mini_task": task_id,
                    "field_expected_feed": expected_feed,
                },
            },
            indent=2,
            ensure_ascii=True,
        )
        + "\n",
        encoding="utf-8",
    )
    (session / "PROMPT.md").write_text(
        "\n".join(
            [
                "# Field Mini Task",
                "",
                f"Field: `{field}`",
                f"Mini task: `{task_id}`",
                "",
                "## Expected Feed",
                expected_feed or "(not provided)",
                "",
            ]
        ),
        encoding="utf-8",
    )
    (session / "FEED.md").write_text("# Current Feed\n\nStatus: partial\n", encoding="utf-8")


def validate_mini_task_schema(field: str, task: object) -> list[str]:
    if not isinstance(task, dict):
        return [f"{field}: mini task must be an object"]
    task_id = task.get("id")
    errors: list[str] = []
    for key in ("id", "goal", "expected_feed"):
        value = task.get(key)
        if not isinstance(value, str) or not value.strip():
            errors.append(f"{field}/{task_id or 'unknown'}: missing non-empty {key}")
    for draft_key in ("query", "field_expected_feed"):
        if draft_key in task:
            errors.append(
                f"{field}/{task_id or 'unknown'}: draft key {draft_key} must not appear in field-pack mini_tasks"
            )
    return errors


def exercise_template(root: Path, template: Path, field: str, task_id: str, expected_feed: str) -> tuple[str | None, str | None]:
    with tempfile.TemporaryDirectory(prefix="octopus-template-check-") as tmp:
        session = Path(tmp) / "session"
        seed_session(session, field, task_id, expected_feed)
        env = {
            "__builtins__": __builtins__,
            "dt": dt,
            "json": json,
            "re": re,
            "sys": sys,
            "Path": Path,
            "compact": compact,
            "rel": rel,
            "root": root,
            "session": session,
            "field": field,
            "mini_task": task_id,
            "expected_feed": expected_feed,
            "query": f"Run {field} mini task {task_id}",
            "field_result": None,
        }
        try:
            source = template.read_text(encoding="utf-8", errors="replace")
            exec(compile(source, str(template), "exec"), env, env)
        except Exception as exc:  # noqa: BLE001 - check report should preserve template errors.
            return f"{field}/{task_id}: execution failed: {type(exc).__name__}: {compact(exc, 360)}", None

        result = env.get("field_result")
        if not isinstance(result, dict):
            return f"{field}/{task_id}: did not set field_result dict", None
        status = result.get("status")
        if status not in {"satisfied", "partial"}:
            return f"{field}/{task_id}: status is {status!r}, expected 'satisfied' or honest 'partial'", None
        metadata = result.get("metadata")
        if not isinstance(metadata, dict):
            return f"{field}/{task_id}: missing metadata dict", None
        if metadata.get("field_pack") != field:
            return f"{field}/{task_id}: metadata.field_pack is {metadata.get('field_pack')!r}", None
        if metadata.get("field_mini_task") != task_id:
            return f"{field}/{task_id}: metadata.field_mini_task is {metadata.get('field_mini_task')!r}", None
        if metadata.get("field_expected_feed") != expected_feed:
            return f"{field}/{task_id}: metadata.field_expected_feed is {metadata.get('field_expected_feed')!r}", None
        evidence = result.get("evidence")
        if not isinstance(evidence, list) or not evidence:
            return f"{field}/{task_id}: missing evidence list", None
        if status == "satisfied" and not any(
            has_artifact_backing(item, root, session) for item in evidence
        ):
            return f"{field}/{task_id}: satisfied result lacks artifact-backed evidence", None
        if status == "partial":
            has_gap = (
                metadata.get("verifier_status") == "partial"
                or bool(metadata.get("error_category"))
                or bool(metadata.get("missing_regions"))
            )
            if not has_gap:
                return f"{field}/{task_id}: partial result must include verifier_status=partial, error_category, or missing_regions", None
    return None, status


def main() -> int:
    cwd = Path.cwd().resolve()
    root = repo_root(cwd)
    tentacle_root_path = (root / "tentacles" / "field-mini-task") if root else tentacle_root(cwd)
    template_root = tentacle_root_path / "repair-templates"
    missing: list[str] = []
    invalid: list[str] = []
    checked: list[str] = []
    executed: list[str] = []
    satisfied: list[str] = []
    partial: list[str] = []

    bundled_pack_root = tentacle_root_path.parent / "field-packs"
    pack_root = (root / "field-packs") if root else bundled_pack_root if bundled_pack_root.exists() else None

    if pack_root:
        index_path = pack_root / "index.json"
        index = json.loads(index_path.read_text(encoding="utf-8"))
        field_tasks: list[tuple[str, str, str, Path]] = []
        fields = index.get("packs") or index.get("fields") or []
        if not fields:
            invalid.append(f"{rel(index_path, pack_root)} has no packs")
        for field in fields:
            pack_path = pack_root / field / "field-pack.json"
            if not pack_path.exists():
                missing.append(f"{field}: missing {rel(pack_path, pack_root)}")
                continue
            pack = json.loads(pack_path.read_text(encoding="utf-8"))
            for task in pack.get("mini_tasks", []):
                invalid.extend(validate_mini_task_schema(field, task))
                task_id = task.get("id", "")
                field_tasks.append(
                    (
                        field,
                        task_id,
                        compact(task.get("expected_feed") or ""),
                        template_root / field / f"{task_id}.pyfrag",
                    )
                )
    else:
        field_tasks = [
            (template.parent.name, template.stem, "", template)
            for template in sorted(template_root.glob("*/*.pyfrag"))
        ]
        if not field_tasks:
            invalid.append(f"{template_root} has no repair templates")

    display_root = root or (pack_root.parent if pack_root else tentacle_root_path)
    for field, task_id, expected_feed, template in field_tasks:
        label = f"{field}/{task_id}"
        if not template.exists():
            missing.append(f"{label}: missing {rel(template, display_root)}")
            continue
        marker = first_nonempty_line(template)
        expected_if = f'if field == "{field}" and mini_task == "{task_id}":'
        if marker != expected_if:
            invalid.append(f"{label}: first code line is {marker!r}")
            continue
        if "elif field ==" in template.read_text(encoding="utf-8", errors="replace"):
            invalid.append(f"{label}: template must be standalone and must not use elif field guard")
            continue
        execution_error, status = exercise_template(display_root, template, field, task_id, expected_feed)
        if execution_error:
            invalid.append(execution_error)
            continue
        checked.append(label)
        executed.append(label)
        if status == "partial":
            partial.append(label)
        else:
            satisfied.append(label)

    if not checked:
        invalid.append("no repair templates were checked")

    report = {
        "status": "ok" if not missing and not invalid else "failed",
        "checked_count": len(checked),
        "executed_count": len(executed),
        "satisfied_count": len(satisfied),
        "partial_count": len(partial),
        "partial": partial,
        "missing": missing,
        "invalid": invalid,
    }
    print(json.dumps(report, indent=2, sort_keys=True))
    return 0 if report["status"] == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())
