#!/usr/bin/env python3
"""BENCH_FIBONACCI_N=35"""

def fib(n: int) -> int:
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)


def main() -> None:
    print(fib(35))


if __name__ == "__main__":
    main()
