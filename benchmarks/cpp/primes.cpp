// BENCH_PRIMES_LIMIT=500000
#include <iostream>

static bool is_prime(int n) {
    if (n < 2) return false;
    for (int d = 2; d * d <= n; ++d) {
        if (n % d == 0) return false;
    }
    return true;
}

static int count_primes(int limit) {
    int count = 0;
    for (int n = 2; n <= limit; ++n) {
        if (is_prime(n)) ++count;
    }
    return count;
}

int main() {
    std::cout << count_primes(500000) << '\n';
    return 0;
}
