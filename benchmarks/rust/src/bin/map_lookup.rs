fn key_at(i: i32) -> i32 {
    i * 3 + 7
}

fn value_at(i: i32) -> i32 {
    i * 31 + 13
}

fn lookup(keys_size: i32, target: i32) -> i32 {
    for i in 0..keys_size {
        if key_at(i) == target {
            return value_at(i);
        }
    }
    -1
}

fn lcg_next(state: i64) -> i64 {
    let s = state * 1_103_515_245 + 12_345;
    if s < 0 { -s } else { s }
}

fn map_workload(map_size: i32, lookups: i32, seed: i32) -> i64 {
    let mut sum = 0_i64;
    let mut state = seed as i64;
    for _ in 0..lookups {
        state = lcg_next(state);
        let idx = (state % map_size as i64) as i32;
        sum += lookup(map_size, key_at(idx)) as i64;
    }
    sum
}

fn main() {
    println!("{}", map_workload(5000, 50_000, 42));
}
