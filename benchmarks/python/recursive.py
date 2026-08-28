#!/usr/bin/env python3
"""BENCH_RECURSIVE_DEPTH=20"""


def recurse(depth: int) -> int:
    if depth == 0:
        return 1
    return recurse(depth - 1) + recurse(depth - 1)


def main() -> None:
    print(recurse(20))


if __name__ == "__main__":
    main()
