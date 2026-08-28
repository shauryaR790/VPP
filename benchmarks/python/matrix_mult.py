#!/usr/bin/env python3
"""BENCH_MATRIX_SIZE=128"""


def mat_value(row: int, col: int, n: int) -> int:
    return (row * 131 + col * 17) % 997


def matrix_multiply(n: int) -> int:
    checksum = 0
    for i in range(n):
        for j in range(n):
            total = 0
            for k in range(n):
                total += mat_value(i, k, n) * mat_value(k, j, n)
            checksum += total
    return checksum


def main() -> None:
    print(matrix_multiply(128))


if __name__ == "__main__":
    main()
