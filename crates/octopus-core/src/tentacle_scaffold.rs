use crate::{capitalize_ascii, OCTOPUS_JSON_CONTRACT};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TentacleScaffold {
    pub tentacle_id: String,
    pub runtime: String,
    pub directory: String,
    pub manifest_path: String,
    pub tool_path: Option<String>,
    pub next_steps: Vec<String>,
}

pub fn scaffold_tentacle(
    workspace_root: impl AsRef<Path>,
    tentacle_id: &str,
    runtime: Option<&str>,
) -> Result<TentacleScaffold, String> {
    let tentacle_id = validate_tentacle_id(tentacle_id)?;
    let runtime = normalize_runtime(runtime)?;

    let tentacles_root = workspace_root.as_ref().join("tentacles");
    let directory = tentacles_root.join(&tentacle_id);
    if directory.exists() {
        return Err(format!("tentacle already exists: {}", directory.display()));
    }
    let tools_dir = directory.join("tools");
    fs::create_dir_all(&tools_dir).map_err(|error| error.to_string())?;
    let schema_path = tentacles_root.join("tentacle.schema.json");
    if !schema_path.exists() {
        fs::write(
            &schema_path,
            include_str!("../../../tentacles/tentacle.schema.json"),
        )
        .map_err(|error| error.to_string())?;
    }

    let (entrypoint, tool_path) = match runtime.as_str() {
        "python" => {
            let path = tools_dir.join("feed.py");
            fs::write(&path, python_scaffold_tool()).map_err(|error| error.to_string())?;
            make_executable(&path)?;
            ("tools/feed.py".to_string(), Some(path))
        }
        "node" => {
            let path = tools_dir.join("feed.js");
            fs::write(&path, node_scaffold_tool()).map_err(|error| error.to_string())?;
            make_executable(&path)?;
            ("tools/feed.js".to_string(), Some(path))
        }
        "shell" => {
            let path = tools_dir.join("feed.sh");
            fs::write(&path, shell_scaffold_tool()).map_err(|error| error.to_string())?;
            make_executable(&path)?;
            ("tools/feed.sh".to_string(), Some(path))
        }
        "http" => ("https://example.com/octopus-feed".to_string(), None),
        _ => ("tools/feed".to_string(), None),
    };

    let manifest_path = directory.join("manifest.json");
    let manifest = scaffold_manifest(&tentacle_id, &runtime, &entrypoint);
    fs::write(
        &manifest_path,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&manifest).map_err(|error| error.to_string())?
        ),
    )
    .map_err(|error| error.to_string())?;

    let mut next_steps = vec![
        format!("review {}", manifest_path.display()),
        format!("octopus manifests {}", tentacles_root.display()),
        format!("octopus --state /tmp/octopus.json install {tentacle_id}"),
    ];
    if tool_path.is_none() && runtime != "http" {
        next_steps.insert(
            1,
            format!(
                "add executable {} that reads {OCTOPUS_JSON_CONTRACT} from stdin",
                directory.join("tools/feed").display()
            ),
        );
    }
    Ok(TentacleScaffold {
        tentacle_id,
        runtime,
        directory: directory.to_string_lossy().to_string(),
        manifest_path: manifest_path.to_string_lossy().to_string(),
        tool_path: tool_path.map(|path| path.to_string_lossy().to_string()),
        next_steps,
    })
}

fn normalize_runtime(runtime: Option<&str>) -> Result<String, String> {
    let value = runtime.unwrap_or("python").trim().to_ascii_lowercase();
    let value = match value.as_str() {
        "javascript" => "node".to_string(),
        "bash" => "shell".to_string(),
        _ => value,
    };
    if value.is_empty() {
        return Err("runtime cannot be empty".to_string());
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
    {
        return Err("runtime can only contain ASCII letters, numbers, '-' and '_'".to_string());
    }
    Ok(value)
}

fn validate_tentacle_id(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("tentacle id cannot be empty".to_string());
    }
    if value.starts_with('.') || value.contains("..") {
        return Err("tentacle id cannot contain path traversal".to_string());
    }
    if !value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '-' || character == '_')
    {
        return Err("tentacle id can only contain ASCII letters, numbers, '-' and '_'".to_string());
    }
    Ok(value.to_string())
}

fn scaffold_manifest(tentacle_id: &str, runtime: &str, entrypoint: &str) -> serde_json::Value {
    serde_json::json!({
        "$schema": "../tentacle.schema.json",
        "schema_version": "0.0.1",
        "id": tentacle_id,
        "name": tentacle_title(tentacle_id),
        "description": "User-owned code-as-harness tentacle.",
        "brain": {
            "kind": "llm",
            "model": null,
            "prompt": "Map a cognitive Need to this tentacle's tool metadata and implementation code. Execute only the needed feed step and return compact evidence.",
            "feedback_contract": "Return status, evidence, touched files or remote calls, and the next useful Need. Do not return hidden reasoning."
        },
        "skills": [
            {
                "id": format!("{tentacle_id}-feed"),
                "description": "Feed observe, execute, and verify Needs through custom harness code.",
                "needs": ["observe", "execute", "verify"]
            }
        ],
        "tools": [
            {
                "id": "feed",
                "description": "Run this tentacle's custom harness implementation.",
                "input": "octopus-json-v1 tool call",
                "output": "structured feedback",
                "implementation": {
                    "kind": runtime,
                    "entrypoint": entrypoint,
                    "contract": OCTOPUS_JSON_CONTRACT
                }
            }
        ],
        "evolution": {
            "editable": ["manifest.json", "brain.prompt", "tools/*"],
            "surfaces": [
                {
                    "id": "brain_prompt",
                    "description": "Tool-side LLM prompt and feedback contract.",
                    "targets": ["brain.prompt", "brain.feedback_contract"]
                },
                {
                    "id": "tool_meta",
                    "description": "Tool descriptions, inputs, outputs, and call contracts.",
                    "targets": ["tools[].description", "tools[].input", "tools[].output", "tools[].permission", "tools[].implementation.contract"]
                },
                {
                    "id": "runtime_code",
                    "description": "Executable harness code owned by the tentacle.",
                    "targets": ["tools/*"]
                },
                {
                    "id": "evolution_policy",
                    "description": "Checks, constraints, and the declared edit surface.",
                    "targets": ["evolution.checks", "evolution.constraints", "evolution.surfaces"]
                }
            ],
            "checks": ["python3 -m json.tool manifest.json > /dev/null"],
            "constraints": ["Keep octopus-json-v1 input and compact feedback output stable."]
        }
    })
}

fn tentacle_title(value: &str) -> String {
    value
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .map(capitalize_ascii)
        .collect::<Vec<_>>()
        .join(" ")
}

fn python_scaffold_tool() -> &'static str {
    r#"#!/usr/bin/env python3
import json
import sys

call = json.load(sys.stdin)
need = call["need"]
tool = call["tool"]
tentacle = call["tentacle"]

print(json.dumps({
    "status": "satisfied",
    "output": f"{need['kind']} feed for {need['query']}",
    "metadata": {
        "tentacle": tentacle["id"],
        "tool": tool["id"],
        "runtime": tool["runtime"],
    },
}))
"#
}

fn node_scaffold_tool() -> &'static str {
    r#"#!/usr/bin/env node
const fs = require("fs");

const call = JSON.parse(fs.readFileSync(0, "utf8"));
const need = call.need;
const tool = call.tool;
const tentacle = call.tentacle;

process.stdout.write(JSON.stringify({
  status: "satisfied",
  output: `${need.kind} feed for ${need.query}`,
  metadata: {
    tentacle: tentacle.id,
    tool: tool.id,
    runtime: tool.runtime
  }
}) + "\n");
"#
}

fn shell_scaffold_tool() -> &'static str {
    r#"#!/usr/bin/env bash
set -euo pipefail

payload="$(cat)"
OCTOPUS_PAYLOAD="$payload" python3 - <<'PY'
import json
import os

call = json.loads(os.environ["OCTOPUS_PAYLOAD"])
need = call["need"]
tool = call["tool"]
tentacle = call["tentacle"]

print(json.dumps({
    "status": "satisfied",
    "output": f"{need['kind']} feed for {need['query']}",
    "metadata": {
        "tentacle": tentacle["id"],
        "tool": tool["id"],
        "runtime": tool["runtime"],
    },
}))
PY
"#
}

fn make_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).map_err(|error| error.to_string())?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}
