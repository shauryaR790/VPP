#!/usr/bin/env python3
"""BENCH_ARRAY_SIZE=5000000"""


def sum_range(n: int) -> int:
    total = 0
    for i in range(n):
        total += i
    return total


def main() -> None:
    print(sum_range(5_000_000))


if __name__ == "__main__":
    main()
