#!/usr/bin/env bash
set -euo pipefail

server="${1:?usage: mcp.sh <server> <tool> [json-args]}"
tool="${2:?usage: mcp.sh <server> <tool> [json-args]}"
args="${3:-{}}"

echo "mcp request"
echo "server=$server"
echo "tool=$tool"
echo "args=$args"
echo "wire this script to your MCP client of choice"

