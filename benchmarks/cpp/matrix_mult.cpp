// BENCH_MATRIX_SIZE=128
#include <iostream>

static int mat_value(int row, int col, int n) {
    (void)n;
    return (row * 131 + col * 17) % 997;
}

static long long matrix_multiply(int n) {
    long long checksum = 0;
    for (int i = 0; i < n; ++i) {
        for (int j = 0; j < n; ++j) {
            long long total = 0;
            for (int k = 0; k < n; ++k) {
                total += static_cast<long long>(mat_value(i, k, n)) * mat_value(k, j, n);
            }
            checksum += total;
        }
    }
    return checksum;
}

int main() {
    std::cout << matrix_multiply(128) << '\n';
    return 0;
}
