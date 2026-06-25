"""Octopus: independent-thinking tools for clean agent brains."""

from .brain import Brain, StaticBrain
from .color import Chromatophore
from .heart import HarnessBeat, Heartbeat, MemoryBeat
from .harness import Harness, Octopus
from .llm import OpenAICompatibleLLM, llm_from_env
from .memory import MemoryRecord, MemoryStore
from .models import Evidence, Feedback, Feed, Need, NeedType, Status
from .planner import LLMPlanner, Plan, PlanningTentacleBrain, RulePlanner, TextLLM, ToolCall
from .router import RouteBook, RouteDecision
from .tentacle import FunctionTentacle, SmartTentacle, Tentacle, TentacleBrain
from .tools import FunctionTool, ShellTool, Tool, ToolResult

__all__ = [
    "Brain",
    "Chromatophore",
    "Evidence",
    "Feedback",
    "Feed",
    "FunctionTentacle",
    "FunctionTool",
    "Harness",
    "HarnessBeat",
    "Heartbeat",
    "LLMPlanner",
    "MemoryBeat",
    "MemoryRecord",
    "MemoryStore",
    "Need",
    "NeedType",
    "Octopus",
    "OpenAICompatibleLLM",
    "Plan",
    "PlanningTentacleBrain",
    "RouteBook",
    "RouteDecision",
    "RulePlanner",
    "ShellTool",
    "SmartTentacle",
    "StaticBrain",
    "Status",
    "Tentacle",
    "TentacleBrain",
    "TextLLM",
    "Tool",
    "ToolCall",
    "ToolResult",
    "llm_from_env",
]
