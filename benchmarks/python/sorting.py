#!/usr/bin/env python3
"""BENCH_SORT_SIZE=2000 BENCH_SORT_SEED=42"""


def rand_at(i: int, seed: int) -> int:
    s = seed + i * 1103515245
    s = abs(s)
    return s % 1_000_000


def sort_kernel(size: int, seed: int) -> int:
    comparisons = 0
    checksum = 0
    for i in range(size):
        for j in range(i + 1, size):
            a = rand_at(i, seed)
            b = rand_at(j, seed)
            comparisons += 1
            if a > b:
                checksum += a - b
            else:
                checksum += b - a
    return comparisons + checksum


def main() -> None:
    print(sort_kernel(2000, 42))


if __name__ == "__main__":
    main()
