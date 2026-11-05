"""JSONL 存储与运行登记。demo 用 JSONL 落盘，实现层按规格切 Parquet。"""

import hashlib
import json
import time
from pathlib import Path


class Store:
    def __init__(self, work: Path):
        self.work = Path(work)
        self.store = self.work / "store"
        self.store.mkdir(parents=True, exist_ok=True)
        self._counter = 0

    # ---- 基础读写 ----

    def read_jsonl(self, name: str) -> list[dict]:
        path = self.store / name
        if not path.exists():
            return []
        return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]

    def write_jsonl(self, name: str, rows: list[dict]) -> None:
        path = self.store / name
        path.write_text("".join(json.dumps(r, ensure_ascii=False) + "\n" for r in rows))

    def append(self, name: str, row: dict) -> None:
        rows = self.read_jsonl(name)
        rows.append(row)
        self.write_jsonl(name, rows)

    # ---- ID 与运行登记 ----

    def _hash(self, seed: str) -> str:
        self._counter += 1
        material = f"{seed}|{time.time_ns()}|{self._counter}"
        return hashlib.sha1(material.encode()).hexdigest()[:8]

    def new_run(self, product: str, region: str, spec_version: str, params: dict) -> str:
        run_id = "run-" + self._hash(f"{product}{region}{spec_version}")
        self.append("runs.jsonl", {
            "run_id": run_id,
            "product": product,
            "region": region,
            "spec_version": spec_version,
            "started_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "params": params,
        })
        return run_id

    def new_id(self, prefix: str, seed: str) -> str:
        return f"{prefix}-" + self._hash(seed)

    # ---- 常用集合 ----

    def questions(self) -> list[dict]:
        return self.read_jsonl("questions.jsonl")

    def keywords(self) -> list[dict]:
        return self.read_jsonl("keywords.jsonl")

    def runs(self) -> list[dict]:
        return self.read_jsonl("runs.jsonl")

    def judgments(self) -> list[dict]:
        return self.read_jsonl("judgments.jsonl")

    def upsert_question(self, q: dict) -> None:
        rows = self.questions()
        for i, row in enumerate(rows):
            if row["question_id"] == q["question_id"]:
                rows[i] = q
                self.write_jsonl("questions.jsonl", rows)
                return
        self.append("questions.jsonl", q)
