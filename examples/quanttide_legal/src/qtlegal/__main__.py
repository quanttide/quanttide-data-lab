"""命令行入口：scripts/qtlegal 或 PYTHONPATH=src python3 -m qtlegal <command>

项目根可用环境变量 QTLEGAL_ROOT 覆盖（测试隔离用）。
"""

import argparse
import os
import sys
from pathlib import Path

from . import commands

ROOT = Path(os.environ.get("QTLEGAL_ROOT", Path(__file__).resolve().parents[2]))


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
