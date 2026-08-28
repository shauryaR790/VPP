// BENCH_FIBONACCI_N=35
#include <iostream>

static long long fib(int n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

int main() {
    std::cout << fib(35) << '\n';
    return 0;
}
