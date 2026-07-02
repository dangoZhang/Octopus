package main

import (
	"encoding/json"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"octopus/field-mini-task/internal/fieldworker"
)

func main() {
	task := fieldworker.LoadTask()
	if _, err := exec.LookPath("go"); err != nil {
		fieldworker.WriteJSON(
			fieldworker.Partial(
				task,
				"Go toolchain unavailable; cannot execute the SWE smoke worker.",
				"go_runtime_missing",
			),
		)
		return
	}

	goVersion, err := exec.Command("go", "version").CombinedOutput()
	if err != nil {
		fieldworker.WriteJSON(
			fieldworker.Partial(
				task,
				"Go runtime probe failed while executing `go version`.",
				"go_runtime_probe_failed",
			),
		)
		return
	}

	ts := time.Now().UTC().Format("20060102T150405Z")
	if task.Workspace == "" {
		task.Workspace = "."
	}

	evidenceDir := filepath.Join(task.Workspace, ".octopus", "field-mini-task", "swe", "swe-go-default-smoke", ts)
	if err := os.MkdirAll(evidenceDir, 0o755); err != nil {
		fieldworker.WriteJSON(
			fieldworker.Partial(
				task,
				"Failed to create SWE smoke artifact directory.",
				"artifact_dir_failure",
			),
		)
		return
	}

	evidencePath := filepath.Join(evidenceDir, "evidence.json")
	evidence := map[string]string{
		"go_version": strings.TrimSpace(string(goVersion)),
		"probe":      "go version",
		"timestamp":  ts,
	}
	payload, err := json.MarshalIndent(evidence, "", "  ")
	if err != nil {
		fieldworker.WriteJSON(
			fieldworker.Partial(
				task,
				"Failed to serialize SWE smoke evidence.",
				"artifact_write_failed",
			),
		)
		return
	}

	if err := os.WriteFile(evidencePath, payload, 0o644); err != nil {
		fieldworker.WriteJSON(
			fieldworker.Partial(
				task,
				"Failed to persist SWE smoke evidence.",
				"artifact_write_failed",
			),
		)
		return
	}

	fieldworker.WriteJSON(
		fieldworker.Satisfied(
			task,
			"Go smoke worker completed successfully. Evidence recorded for swe-go-default-smoke.",
			evidencePath,
		),
	)
}
