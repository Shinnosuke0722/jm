#!/usr/bin/env python3
"""Capture eight-process local-list batches as machine-readable benchmark data."""

import json
import os
import subprocess
import time
from pathlib import Path


root = Path(os.environ["BENCH_ROOT"]).resolve()
out = Path(os.environ["RESULTS_DIR"]).resolve() / "concurrency-n100.json"
fixture = root / "fixtures" / "n100"
commands = {
    "jm": [str(root / "bin" / "jm"), "list", "--no-color"],
    "jabba": [str(root / "bin" / "jabba"), "ls"],
    "mise": [str(root / "bin" / "mise"), "ls", "--installed", "java", "--no-header"],
}


def environment(tool: str) -> dict[str, str]:
    env = os.environ.copy()
    if tool == "jm":
        env["JM_HOME"] = str(fixture / "jm")
    elif tool == "jabba":
        env["JABBA_HOME"] = str(fixture / "jabba")
    else:
        env.update({
            "MISE_DATA_DIR": str(fixture / "mise-data"),
            "MISE_CONFIG_DIR": str(fixture / "mise-config"),
            "MISE_CACHE_DIR": str(fixture / "mise-cache"),
            "MISE_STATE_DIR": str(fixture / "mise-state"),
            "MISE_NO_AUTO_INSTALL": "1",
        })
    return env


def batch(tool: str) -> dict[str, float | int]:
    started = time.perf_counter()
    processes = [subprocess.Popen(commands[tool], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, env=environment(tool)) for _ in range(8)]
    statuses = [process.wait() for process in processes]
    elapsed = time.perf_counter() - started
    if any(statuses):
        raise RuntimeError(f"{tool} batch failed: {statuses}")
    return {"seconds": elapsed, "throughput_commands_per_second": 8 / elapsed}


data: dict[str, object] = {"warmups": 2, "runs": 12, "tools": {}}
for tool in commands:
    for _ in range(2):
        batch(tool)
    data["tools"][tool] = [batch(tool) for _ in range(12)]

out.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
