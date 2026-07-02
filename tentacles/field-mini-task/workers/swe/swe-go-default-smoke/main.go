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
		failureTS := time.Now().UTC().Format("20060102T150405Z")
		failureWorkspace := task.Workspace
		if failureWorkspace == "" {
			failureWorkspace = "."
		}
		failureEvidenceDir := filepath.Join(
			failureWorkspace,
			".octopus",
			"field-mini-task",
			"swe",
			"swe-go-default-smoke",
			failureTS,
		)
		if err := os.MkdirAll(failureEvidenceDir, 0o755); err != nil {
			fieldworker.WriteJSON(
				fieldworker.Partial(
					task,
					"Failed to create SWE smoke artifact directory during runtime probe.",
					"artifact_dir_failure",
				),
			)
			return
		}
		failureEvidencePath := filepath.Join(failureEvidenceDir, "evidence.json")
		failureEvidence := map[string]string{
			"timestamp":          failureTS,
			"field_pack":         task.Field,
			"field_mini_task":    task.MiniTask,
			"field_expected_feed": task.ExpectedFeed,
			"error_category":     "go_runtime_missing",
			"probe":              "go executable",
			"probe_error":        err.Error(),
		}
		payload, marshalErr := json.MarshalIndent(failureEvidence, "", "  ")
		if marshalErr != nil {
			fieldworker.WriteJSON(
				fieldworker.Partial(
					task,
					"Failed to serialize SWE smoke evidence.",
					"artifact_write_failed",
				),
			)
			return
		}
		if err := os.WriteFile(failureEvidencePath, payload, 0o644); err != nil {
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
			fieldworker.Partial(
				task,
				"Go toolchain unavailable; cannot execute the SWE smoke worker. Evidence written to: "+failureEvidencePath,
				"go_runtime_missing",
			),
		)
		return
	}

	goVersion, err := exec.Command("go", "version").CombinedOutput()
	if err != nil {
		probeTS := time.Now().UTC().Format("20060102T150405Z")
		probeWorkspace := task.Workspace
		if probeWorkspace == "" {
			probeWorkspace = "."
		}
		probeEvidenceDir := filepath.Join(
			probeWorkspace,
			".octopus",
			"field-mini-task",
			"swe",
			"swe-go-default-smoke",
			probeTS,
		)
		if err := os.MkdirAll(probeEvidenceDir, 0o755); err != nil {
			fieldworker.WriteJSON(
				fieldworker.Partial(
					task,
					"Failed to create SWE smoke artifact directory during runtime probe.",
					"artifact_dir_failure",
				),
			)
			return
		}
		probeEvidencePath := filepath.Join(probeEvidenceDir, "evidence.json")
		probeEvidence := map[string]string{
			"timestamp":          probeTS,
			"field_pack":         task.Field,
			"field_mini_task":    task.MiniTask,
			"field_expected_feed": task.ExpectedFeed,
			"error_category":     "go_runtime_probe_failed",
			"probe":              "go version",
			"go_version":         strings.TrimSpace(string(goVersion)),
		}
		payload, marshalErr := json.MarshalIndent(probeEvidence, "", "  ")
		if marshalErr != nil {
			fieldworker.WriteJSON(
				fieldworker.Partial(
					task,
					"Failed to serialize SWE smoke evidence.",
					"artifact_write_failed",
				),
			)
			return
		}
		if err := os.WriteFile(probeEvidencePath, payload, 0o644); err != nil {
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
			fieldworker.Partial(
				task,
				"Go runtime probe failed while executing `go version`. Evidence written to: "+probeEvidencePath,
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
