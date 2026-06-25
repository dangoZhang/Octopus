from __future__ import annotations

import json
from dataclasses import dataclass, field
from typing import Protocol

from .models import Evidence, Feed, Need, Status
from .tools import Tool, ToolResult


@dataclass(frozen=True)
class ToolCall:
    tool: str
    reason: str
    payload: dict[str, str] = field(default_factory=dict)


@dataclass(frozen=True)
class Plan:
    calls: tuple[ToolCall, ...]
    summary: str


class Planner(Protocol):
    def plan(self, need: Need, tools: tuple[Tool, ...]) -> Plan:
        ...


class TextLLM(Protocol):
    def complete(self, system: str, user: str) -> str:
        ...


class RulePlanner:
    """Fast default planner for tool-side thinking."""

    def plan(self, need: Need, tools: tuple[Tool, ...]) -> Plan:
        calls = tuple(
            ToolCall(tool.name, f"{tool.name} supports {need.kind.value}", {"query": need.query})
            for tool in tools
            if tool.supports(need)
        )
        return Plan(calls[:1], "selected the first matching tool" if calls else "no matching tool")


class LLMPlanner:
    """LLM planner for tentacles. It falls back to rules if JSON is invalid."""

    def __init__(self, llm: TextLLM, fallback: Planner | None = None):
        self.llm = llm
        self.fallback = fallback or RulePlanner()

    def plan(self, need: Need, tools: tuple[Tool, ...]) -> Plan:
        tool_sheet = "\n".join(f"- {tool.name}: {tool.description}" for tool in tools)
        prompt = (
            "Return JSON: {\"calls\":[{\"tool\":\"name\",\"reason\":\"short\"}],"
            "\"summary\":\"short\"}.\n"
            f"Need: {need.kind.value}: {need.query}\nTools:\n{tool_sheet}"
        )
        raw = self.llm.complete(
            "You are a tentacle brain. Choose tools for the need. Do not solve unrelated tasks.",
            prompt,
        )
        try:
            data = json.loads(raw)
            calls = tuple(
                ToolCall(str(item["tool"]), str(item.get("reason", "")))
                for item in data.get("calls", [])
            )
            return Plan(calls, str(data.get("summary", "planned by llm")))
        except Exception:
            return self.fallback.plan(need, tools)


class PlanningTentacleBrain:
    """Executes needs through a planner and tool set."""

    def __init__(self, tools: tuple[Tool, ...], planner: Planner | None = None):
        self.tools = {tool.name: tool for tool in tools}
        self.planner = planner or RulePlanner()

    def handle(self, need: Need) -> Feed:
        plan = self.planner.plan(need, tuple(self.tools.values()))
        if not plan.calls:
            return Feed.unsupported(need, plan.summary)

        results: list[ToolResult] = []
        for call in plan.calls:
            tool = self.tools.get(call.tool)
            if tool is None:
                results.append(ToolResult.failed(f"unknown tool: {call.tool}", tool=call.tool))
                continue
            results.append(tool.run(need))

        status = self._status(tuple(results))
        output = "\n".join(str(result.output) for result in results if str(result.output)).strip()
        evidence = tuple(item for result in results for item in result.evidence)
        metadata = {"plan": plan.summary, "tools": [call.tool for call in plan.calls]}
        return Feed(need, status, evidence or (Evidence("plan", plan.summary, 0.4),), output, metadata)

    def _status(self, results: tuple[ToolResult, ...]) -> Status:
        statuses = {result.status for result in results}
        if statuses == {Status.SATISFIED}:
            return Status.SATISFIED
        if Status.SATISFIED in statuses:
            return Status.PARTIAL
        return Status.FAILED

