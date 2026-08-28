// BENCH_MAP_SIZE=5000 BENCH_MAP_LOOKUPS=50000
#include <cstdlib>
#include <iostream>

static int key_at(int i) { return i * 3 + 7; }
static int value_at(int i) { return i * 31 + 13; }

static int lookup(int keys_size, int target) {
    for (int i = 0; i < keys_size; ++i) {
        if (key_at(i) == target) return value_at(i);
    }
    return -1;
}

static long long lcg_next(long long state) {
    long long s = state * 1103515245LL + 12345LL;
    return s < 0 ? -s : s;
}

static long long map_workload(int map_size, int lookups, int seed) {
    long long sum = 0;
    long long state = seed;
    for (int n = 0; n < lookups; ++n) {
        state = lcg_next(state);
        int idx = static_cast<int>(state % map_size);
        sum += lookup(map_size, key_at(idx));
    }
    return sum;
}

int main() {
    std::cout << map_workload(5000, 50000, 42) << '\n';
    return 0;
}
