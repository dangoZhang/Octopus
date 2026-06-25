from __future__ import annotations

import argparse
import json
from pathlib import Path

from .harness import Harness
from .models import Need, NeedType


DEFAULT_STATE = Path.home() / ".octopus" / "state.json"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="octopus-py", description="Independent-thinking tools, clean brain")
    parser.add_argument("--state", default=str(DEFAULT_STATE), help="persistent harness state file")
    sub = parser.add_subparsers(dest="command", required=True)

    need = sub.add_parser("need", help="feed one cognitive need")
    need.add_argument("kind", choices=[item.value for item in NeedType])
    need.add_argument("query")
    need.add_argument("--json", action="store_true", help="print machine-readable feedback")

    sub.add_parser("routes", help="show learned harness routes")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    harness = Harness.load(args.state)

    if args.command == "routes":
        print(json.dumps(harness.routes, ensure_ascii=False, indent=2))
        return 0

    feedback = harness.feed(Need(NeedType(args.kind), args.query))
    harness.save(args.state)
    if args.json:
        print(
            json.dumps(
                {
                    "status": feedback.status.value,
                    "summary": feedback.summary,
                    "signals": feedback.signals,
                },
                ensure_ascii=False,
                indent=2,
            )
        )
    else:
        print(feedback.summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
