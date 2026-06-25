import unittest

from octopus import FunctionTentacle, Harness, Need, NeedType, Octopus, StaticBrain, Status


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

    def test_octopus_pulse_returns_feedback_to_brain(self):
        brain = StaticBrain([Need.remember("three hearts")])
        octopus = Octopus(brain)

        feedback = octopus.pulse()

        self.assertEqual(feedback.status, Status.SATISFIED)
        self.assertEqual(brain.feedback, (feedback,))


if __name__ == "__main__":
    unittest.main()
