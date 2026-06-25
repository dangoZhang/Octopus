import unittest
from tempfile import TemporaryDirectory

from octopus import (
    FunctionTentacle,
    FunctionTool,
    Harness,
    Need,
    NeedType,
    Octopus,
    PlanningTentacleBrain,
    SmartTentacle,
    StaticBrain,
    Status,
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


if __name__ == "__main__":
    unittest.main()
