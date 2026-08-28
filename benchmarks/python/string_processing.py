#!/usr/bin/env python3
"""BENCH_STRING_ITERS=3000 BENCH_STRING_CHUNK=benchmark-chunk-0123456789"""


def build_string(iters: int, chunk: str) -> int:
    out = ""
    for _ in range(iters):
        out += chunk
    return len(out)


def main() -> None:
    print(build_string(3000, "benchmark-chunk-0123456789"))


if __name__ == "__main__":
    main()
