#!/usr/bin/env python3
"""BENCH_FILE_LINES=5000"""
from pathlib import Path


def file_workload(path: Path, lines: int) -> int:
    body = "".join("benchmark-line\n" for _ in range(lines))
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(body, encoding="utf-8")
    return len(path.read_text(encoding="utf-8"))


def main() -> None:
    target = Path("benchmarks/results/tmp/bench-io.txt")
    print(file_workload(target, 5000))


if __name__ == "__main__":
    main()
