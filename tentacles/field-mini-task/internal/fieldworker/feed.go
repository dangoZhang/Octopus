package fieldworker

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"
)

type Task struct {
	Workspace    string
	Session      string
	Field        string
	MiniTask     string
	ExpectedFeed string
	NeedQuery    string
	ContextJSON  string
}

type Evidence struct {
	Source     string            `json:"source"`
	Content    string            `json:"content"`
	Confidence float64           `json:"confidence"`
	Metadata   map[string]string `json:"metadata"`
}

type Feed struct {
	Status   string            `json:"status"`
	Output   string            `json:"output"`
	Evidence []Evidence        `json:"evidence"`
	Metadata map[string]string `json:"metadata"`
}

func LoadTask() Task {
	return Task{
		Workspace:    os.Getenv("OCTOPUS_WORKSPACE"),
		Session:      os.Getenv("OCTOPUS_SESSION"),
		Field:        os.Getenv("OCTOPUS_FIELD"),
		MiniTask:     os.Getenv("OCTOPUS_MINI_TASK"),
		ExpectedFeed: os.Getenv("OCTOPUS_EXPECTED_FEED"),
		NeedQuery:    os.Getenv("OCTOPUS_NEED_QUERY"),
		ContextJSON:  os.Getenv("OCTOPUS_CONTEXT_JSON"),
	}
}

func BaseMetadata(task Task) map[string]string {
	return map[string]string{
		"runtime":             "go",
		"runtime_template":    "go-worker",
		"tool":                "run_field_mini_task",
		"tentacle":            "field-mini-task",
		"field_pack":          task.Field,
		"field_mini_task":     task.MiniTask,
		"field_expected_feed": task.ExpectedFeed,
		"field_session":       task.Session,
		"verifier_status":     "partial",
	}
}

func Partial(task Task, summary string, errorCategory string) Feed {
	summary = compact(summary, 720)
	metadata := BaseMetadata(task)
	metadata["error_category"] = errorCategory
	metadata["next_need_kind"] = "execute"
	metadata["next_need_query"] = fmt.Sprintf("evolve Go worker harness for %s after %s", task.Field, task.MiniTask)
	return Feed{
		Status: "partial",
		Output: summary,
		Evidence: []Evidence{{
			Source:     "field-mini-task/go-worker",
			Content:    summary,
			Confidence: 0.76,
			Metadata:   metadata,
		}},
		Metadata: metadata,
	}
}

func Satisfied(task Task, summary string, artifactPath string) Feed {
	summary = compact(summary, 720)
	metadata := BaseMetadata(task)
	metadata["verifier_status"] = "satisfied"
	metadata["artifact_path"] = artifactPath
	return Feed{
		Status: "satisfied",
		Output: summary,
		Evidence: []Evidence{{
			Source:     "field-mini-task/go-worker",
			Content:    artifactPath,
			Confidence: 0.88,
			Metadata:   metadata,
		}},
		Metadata: metadata,
	}
}

func WriteJSON(feed Feed) {
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetEscapeHTML(false)
	if err := encoder.Encode(feed); err != nil {
		fmt.Fprintf(os.Stderr, "write feed json: %v\n", err)
		os.Exit(2)
	}
}

func compact(value string, limit int) string {
	text := strings.Join(strings.Fields(value), " ")
	if len(text) <= limit {
		return text
	}
	if limit <= 3 {
		return text[:limit]
	}
	return text[:limit-3] + "..."
}
