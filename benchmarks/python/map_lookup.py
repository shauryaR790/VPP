#!/usr/bin/env python3
"""BENCH_MAP_SIZE=5000 BENCH_MAP_LOOKUPS=50000 BENCH_SORT_SEED=42"""


def key_at(i: int) -> int:
    return i * 3 + 7


def value_at(i: int) -> int:
    return i * 31 + 13


def lookup(keys_size: int, target: int) -> int:
    for i in range(keys_size):
        if key_at(i) == target:
            return value_at(i)
    return -1


def lcg_next(state: int) -> int:
    return abs(state * 1103515245 + 12345)


def map_workload(map_size: int, lookups: int, seed: int) -> int:
    total = 0
    state = seed
    for _ in range(lookups):
        state = lcg_next(state)
        idx = state % map_size
        key = key_at(idx)
        total += lookup(map_size, key)
    return total


def main() -> None:
    print(map_workload(5000, 50000, 42))


if __name__ == "__main__":
    main()
