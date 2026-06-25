use octopus_core::{
    FunctionTool, Harness, Need, NeedKind, PlanningTentacle, RulePlanner, ToolResult,
};

fn main() {
    let verifier = FunctionTool::new(
        "verifier",
        "checks claims and returns compact evidence",
        vec![NeedKind::Verify],
        |need| ToolResult::satisfied("verifier", format!("verified: {}", need.query)),
    );
    let research = PlanningTentacle::new(
        "research",
        vec![NeedKind::Verify],
        RulePlanner,
        vec![Box::new(verifier)],
    );
    let mut harness = Harness::new();
    harness.add_tentacle(Box::new(research));

    let feed = harness.feed_one(&Need::new(
        NeedKind::Verify,
        "the brain does not name tools",
    ));
    println!("{}", feed.summary);
    println!("plan: {}", feed.metadata["plan"]);
}
