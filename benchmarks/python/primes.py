#!/usr/bin/env python3
"""BENCH_PRIMES_LIMIT=500000"""


def is_prime(n: int) -> bool:
    if n < 2:
        return False
    d = 2
    while d * d <= n:
        if n % d == 0:
            return False
        d += 1
    return True


def count_primes(limit: int) -> int:
    count = 0
    for n in range(2, limit + 1):
        if is_prime(n):
            count += 1
    return count


def main() -> None:
    print(count_primes(500000))


if __name__ == "__main__":
    main()
