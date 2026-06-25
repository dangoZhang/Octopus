# Research Map

Octopus is not a paper clone. It uses these works as pressure tests for the architecture.

## Tool Use

- [ReAct: Synergizing Reasoning and Acting in Language Models](https://arxiv.org/abs/2210.03629): shows the value of combining reasoning and environment actions. Octopus keeps the main brain lighter by moving concrete action work into tentacles.
- [Toolformer: Language Models Can Teach Themselves to Use Tools](https://arxiv.org/abs/2302.04761): motivates learned tool-use decisions. Octopus makes tool choice a harness/tentacle concern rather than a fixed brain loop.
- [Gorilla: Large Language Model Connected with Massive APIs](https://arxiv.org/abs/2305.15334): highlights API accuracy and retrieval over changing tool docs. Octopus treats tool execution as an intelligent tentacle surface.
- [ToolLLM: Facilitating Large Language Models to Master 16000+ Real-world APIs](https://arxiv.org/abs/2307.16789): shows scale pressure for real API use. Octopus keeps the core contract small so many tool systems can attach.

## Learning From Feedback

- [Reflexion: Language Agents with Verbal Reinforcement Learning](https://arxiv.org/abs/2303.11366): uses feedback and memory to improve later trials. Octopus persists memory and route scores in the harness state.
- [Voyager: An Open-Ended Embodied Agent with Large Language Models](https://arxiv.org/abs/2305.16291): demonstrates executable skill growth from environment feedback. Octopus separates skill/tool growth from the clean brain.

## Memory And Retrieval

- [MemGPT: Towards LLMs as Operating Systems](https://arxiv.org/abs/2310.08560): frames context as managed memory. Octopus keeps memory as a harness-evolved environment beat.
- [Self-RAG: Learning to Retrieve, Generate, and Critique through Self-Reflection](https://arxiv.org/abs/2310.11511): motivates demand-driven retrieval and critique. Octopus expresses demand as Need and feeds evidence back through Feedback.

## Design Bet

The core bet is DataDriven cognitive supply: once the brain emits a need, the harness should learn how to satisfy it with the best available feed.
