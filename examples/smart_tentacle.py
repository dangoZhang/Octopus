from octopus import FunctionTool, Harness, Need, NeedType, PlanningTentacleBrain, SmartTentacle


def main() -> None:
    tool = FunctionTool(
        "verifier",
        "checks a claim and returns compact evidence",
        (NeedType.VERIFY,),
        lambda need: f"verified: {need.query}",
    )
    tentacle_brain = PlanningTentacleBrain((tool,))
    harness = Harness()
    harness.add_tentacle(SmartTentacle("research", [NeedType.VERIFY], tentacle_brain))

    feedback = harness.feed(Need.verify("Need should not name tools."))
    print(feedback.summary)


if __name__ == "__main__":
    main()

