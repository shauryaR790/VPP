// BENCH_SORT_SIZE=2000 BENCH_SORT_SEED=42
#include <cstdlib>
#include <iostream>

static int rand_at(int i, int seed) {
    long long s = static_cast<long long>(seed) + static_cast<long long>(i) * 1103515245LL;
    if (s < 0) s = -s;
    return static_cast<int>(s % 1000000LL);
}

static long long sort_kernel(int size, int seed) {
    long long comparisons = 0;
    long long checksum = 0;
    for (int i = 0; i < size; ++i) {
        for (int j = i + 1; j < size; ++j) {
            int a = rand_at(i, seed);
            int b = rand_at(j, seed);
            ++comparisons;
            if (a > b) checksum += a - b;
            else checksum += b - a;
        }
    }
    return comparisons + checksum;
}

int main() {
    std::cout << sort_kernel(2000, 42) << '\n';
    return 0;
}
