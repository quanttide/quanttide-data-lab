"""命令行入口：python -m qtlegal <command>"""

import argparse
import sys
from pathlib import Path

from . import commands

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    parser = argparse.ArgumentParser(prog="qtlegal", description="算法备案数据集四层管道 demo")
    sub = parser.add_subparsers(dest="command", required=True)
    for name, help_text in [
        ("spec", "规格书校验与版本化"),
        ("collect", "词库采集与登记"),
        ("generate", "生产 draft（入库即拦截）"),
        ("judge", "研判双人流（draft → judged → verified）"),
        ("verify", "覆盖差集、量纲、溯源与指标计算"),
        ("deliver", "打包交付与报告生成"),
        ("demo", "按序运行全链"),
    ]:
        sub.add_parser(name, help=help_text)

    args = parser.parse_args()
    if args.command == "demo":
        for step in ("spec", "collect", "generate", "judge", "verify", "deliver"):
            print(f"===== {step} =====")
            code = getattr(commands, f"cmd_{step}")(ROOT)
            if code != 0:
                print(f"===== {step} 未过，链路终止 =====")
                return code
        return 0
    return getattr(commands, f"cmd_{args.command}")(ROOT)


if __name__ == "__main__":
    sys.exit(main())
