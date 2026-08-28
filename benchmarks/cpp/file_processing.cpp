// BENCH_FILE_LINES=5000
#include <fstream>
#include <iostream>
#include <string>

static int file_workload(const std::string& path, int lines) {
    std::string body;
    body.reserve(static_cast<size_t>(lines) * 15);
    for (int i = 0; i < lines; ++i) body += "benchmark-line\n";
    {
        std::ofstream out(path, std::ios::binary | std::ios::trunc);
        out << body;
    }
    std::ifstream in(path, std::ios::binary);
    return static_cast<int>(std::string((std::istreambuf_iterator<char>(in)), std::istreambuf_iterator<char>()).size());
}

int main() {
    std::cout << file_workload("benchmarks/results/tmp/bench-io.txt", 5000) << '\n';
    return 0;
}
