import unittest
from tempfile import TemporaryDirectory
from unittest.mock import patch

from octopus import (
    FunctionTentacle,
    FunctionTool,
    Harness,
    LLMPlanner,
    Need,
    NeedType,
    Octopus,
    OpenAICompatibleLLM,
    PlanningTentacleBrain,
    SmartTentacle,
    StaticBrain,
    Status,
    llm_from_env,
)


class OctopusTest(unittest.TestCase):
    def test_brain_emits_needs_without_tools(self):
        brain = StaticBrain([Need.verify("check the claim")])
        self.assertEqual([need.kind for need in brain.needs()], [NeedType.VERIFY])

    def test_harness_routes_need_to_tentacle(self):
        harness = Harness()
        harness.add_tentacle(FunctionTentacle("verifier", [NeedType.VERIFY], lambda need: "verified"))

        feedback = harness.feed(Need.verify("claim"))

        self.assertEqual(feedback.status, Status.SATISFIED)
        self.assertEqual(feedback.summary, "verified")
        self.assertEqual(feedback.feeds[0].metadata["tentacle"], "verifier")

    def test_memory_is_a_tentacle_not_brain_state(self):
        harness = Harness()

        harness.feed(Need.remember("Need and implementation are split."))
        feedback = harness.feed(Need.recall("implementation"))

        self.assertIn("implementation are split", feedback.summary)

    def test_memory_recall_ignores_unmatched_weighted_records(self):
        harness = Harness()

        harness.feed(Need.remember("alpha route"))
        harness.feed(Need.remember("beta tentacle"))
        feedback = harness.feed(Need.recall("tentacle"))

        self.assertEqual(feedback.summary, "beta tentacle")

    def test_octopus_pulse_returns_feedback_to_brain(self):
        brain = StaticBrain([Need.remember("three hearts")])
        octopus = Octopus(brain)

        feedback = octopus.pulse()

        self.assertEqual(feedback.status, Status.SATISFIED)
        self.assertEqual(brain.feedback, (feedback,))

    def test_harness_learns_successful_route(self):
        harness = Harness()
        harness.add_tentacle(FunctionTentacle("first", [NeedType.VERIFY], lambda need: "ok"))

        feedback = harness.feed(Need.verify("claim"))

        self.assertEqual(feedback.status, Status.SATISFIED)
        self.assertGreater(harness.routes["verify:first"], 1.0)

    def test_smart_tentacle_thinks_before_tool_execution(self):
        tool = FunctionTool(
            "echo",
            "echoes the need query",
            (NeedType.EXECUTE,),
            lambda need: f"ran {need.query}",
        )
        brain = PlanningTentacleBrain((tool,))
        harness = Harness()
        harness.add_tentacle(SmartTentacle("executor", [NeedType.EXECUTE], brain))

        feedback = harness.feed(Need.execute("task"))

        self.assertEqual(feedback.status, Status.SATISFIED)
        self.assertEqual(feedback.summary, "ran task")
        self.assertEqual(feedback.feeds[0].metadata["tools"], ["echo"])

    def test_harness_state_persists_memory_and_routes(self):
        with TemporaryDirectory() as directory:
            path = f"{directory}/state.json"
            harness = Harness()
            harness.feed(Need.remember("persistent feed"))
            harness.save(path)

            restored = Harness.load(path)
            feedback = restored.feed(Need.recall("persistent"))

            self.assertIn("persistent feed", feedback.summary)
            self.assertGreater(restored.routes["recall:recall"], 1.0)

    def test_openai_compatible_llm_posts_chat_completion(self):
        captured = {}

        class Response:
            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

            def read(self):
                return b'{"choices":[{"message":{"content":"{\\"calls\\":[],\\"summary\\":\\"ok\\"}"}}]}'

        def fake_urlopen(request, timeout):
            captured["url"] = request.full_url
            captured["timeout"] = timeout
            captured["headers"] = dict(request.header_items())
            captured["body"] = request.data
            return Response()

        llm = OpenAICompatibleLLM(
            model="test-model",
            api_key="token",
            base_url="https://llm.example/v1/",
            timeout=3,
        )

        with patch("urllib.request.urlopen", fake_urlopen):
            content = llm.complete("system", "user")

        body = captured["body"].decode("utf-8")
        self.assertEqual(captured["url"], "https://llm.example/v1/chat/completions")
        self.assertEqual(captured["timeout"], 3)
        self.assertIn(("Authorization", "Bearer token"), captured["headers"].items())
        self.assertIn('"model": "test-model"', body)
        self.assertIn('"role": "system"', body)
        self.assertIn('"role": "user"', body)
        self.assertIn('"summary":"ok"', content)

    def test_llm_from_env_builds_openai_compatible_adapter(self):
        env = {
            "OCTOPUS_LLM_MODEL": "local-model",
            "OCTOPUS_LLM_BASE_URL": "http://localhost:11434/v1",
            "OCTOPUS_LLM_API_KEY": "",
            "OCTOPUS_LLM_TIMEOUT": "2",
        }
        with patch.dict("os.environ", env, clear=False):
            llm = llm_from_env()

        self.assertEqual(llm.model, "local-model")
        self.assertEqual(llm.base_url, "http://localhost:11434/v1")
        self.assertEqual(llm.timeout, 2)

    def test_llm_planner_accepts_openai_compatible_adapter(self):
        class Response:
            def __enter__(self):
                return self

            def __exit__(self, exc_type, exc, tb):
                return False

            def read(self):
                return (
                    b'{"choices":[{"message":{"content":'
                    b'"{\\"calls\\":[{\\"tool\\":\\"echo\\",\\"reason\\":\\"match\\"}],'
                    b'\\"summary\\":\\"planned by provider\\"}"}}]}'
                )

        tool = FunctionTool(
            "echo",
            "echoes the need query",
            (NeedType.EXECUTE,),
            lambda need: f"ran {need.query}",
        )
        planner = LLMPlanner(OpenAICompatibleLLM(model="test-model"))

        with patch("urllib.request.urlopen", lambda request, timeout: Response()):
            plan = planner.plan(Need.execute("task"), (tool,))

        self.assertEqual(plan.summary, "planned by provider")
        self.assertEqual(plan.calls[0].tool, "echo")


if __name__ == "__main__":
    unittest.main()
