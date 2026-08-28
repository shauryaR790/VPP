// BENCH_ARITHMETIC_ITERS=500000000
#include <iostream>

static long long arithmetic(long long iters) {
    long long x = 1;
    for (long long i = 0; i < iters; ++i) {
        x = x + (x * 3) % 1000003;
        if (x < 0) x = -x;
        x = x - (x / 4);
    }
    return x;
}

int main() {
    std::cout << arithmetic(500000000LL) << '\n';
    return 0;
}
