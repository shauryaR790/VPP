// BENCH_RECURSIVE_DEPTH=20
#include <iostream>

static int recurse(int depth) {
    if (depth == 0) return 1;
    return recurse(depth - 1) + recurse(depth - 1);
}

int main() {
    std::cout << recurse(20) << '\n';
    return 0;
}
