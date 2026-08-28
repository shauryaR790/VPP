// BENCH_ARRAY_SIZE=5000000
#include <iostream>

static long long sum_range(int n) {
    long long total = 0;
    for (int i = 0; i < n; ++i) total += i;
    return total;
}

int main() {
    std::cout << sum_range(5000000) << '\n';
    return 0;
}
