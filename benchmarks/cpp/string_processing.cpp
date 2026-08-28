// BENCH_STRING_ITERS=3000
#include <iostream>
#include <string>

static int build_string(int iters, const std::string& chunk) {
    std::string out;
    out.reserve(chunk.size() * static_cast<size_t>(iters));
    for (int i = 0; i < iters; ++i) out += chunk;
    return static_cast<int>(out.size());
}

int main() {
    std::cout << build_string(3000, "benchmark-chunk-0123456789") << '\n';
    return 0;
}
