#!/usr/bin/env bash
set -euo pipefail

server="${1:?usage: mcp.sh <server> <tool> [json-args]}"
tool="${2:?usage: mcp.sh <server> <tool> [json-args]}"
if [ "$#" -ge 3 ]; then
  args="$3"
else
  args="{}"
fi

server_key="$(printf '%s' "$server" | tr '[:lower:]' '[:upper:]' | sed 's/[^A-Z0-9_]/_/g')"
server_command_var="OCTOPUS_MCP_${server_key}_COMMAND"
client_command="${!server_command_var:-${OCTOPUS_MCP_COMMAND:-}}"

if command -v python3 >/dev/null 2>&1; then
  request="$(python3 - "$server" "$tool" "$args" <<'PY'
import json
import sys

server, tool, raw_args = sys.argv[1:]
try:
    parsed_args = json.loads(raw_args)
except json.JSONDecodeError as error:
    raise SystemExit(f"invalid MCP JSON args: {error}")

print(json.dumps({
    "jsonrpc": "2.0",
    "id": "octopus-mcp-call",
    "method": "tools/call",
    "params": {
        "name": tool,
        "arguments": parsed_args,
    },
}, separators=(",", ":")))
PY
)"
else
  request="{\"jsonrpc\":\"2.0\",\"id\":\"octopus-mcp-call\",\"method\":\"tools/call\",\"params\":{\"name\":\"$tool\",\"arguments\":$args}}"
fi

if [ -z "$client_command" ]; then
  if command -v python3 >/dev/null 2>&1; then
    python3 - "$server" "$tool" "$args" "$request" "$server_command_var" <<'PY'
import json
import sys

server, tool, raw_args, request, server_command_var = sys.argv[1:]
print(json.dumps({
    "error": "mcp_not_configured",
    "server": server,
    "tool": tool,
    "args": raw_args,
    "request": json.loads(request),
    "hint": f"set {server_command_var} or OCTOPUS_MCP_COMMAND to an MCP client command that reads JSON-RPC on stdin",
}, separators=(",", ":")))
PY
  else
    echo "mcp_not_configured"
    echo "server=$server"
    echo "tool=$tool"
    echo "args=$args"
    echo "request=$request"
    echo "set $server_command_var or OCTOPUS_MCP_COMMAND to an MCP client command that reads JSON-RPC on stdin"
  fi
  exit 1
fi

printf '%s\n' "$request" | OCTOPUS_MCP_SERVER="$server" OCTOPUS_MCP_TOOL="$tool" sh -c "$client_command"
