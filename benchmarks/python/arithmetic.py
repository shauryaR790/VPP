#!/usr/bin/env python3
"""BENCH_ARITHMETIC_ITERS=500000000"""


def arithmetic(iters: int) -> int:
    x = 1
    for _ in range(iters):
        x = x + (x * 3) % 1_000_003
        if x < 0:
            x = -x
        x = x - (x // 4)
    return x


def main() -> None:
    print(arithmetic(500_000_000))


if __name__ == "__main__":
    main()
