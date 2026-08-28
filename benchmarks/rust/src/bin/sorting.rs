fn rand_at(i: i32, seed: i32) -> i32 {
    let mut s = seed as i64 + i as i64 * 1_103_515_245;
    if s < 0 {
        s = -s;
    }
    (s % 1_000_000) as i32
}

fn sort_kernel(size: i32, seed: i32) -> i64 {
    let mut comparisons = 0_i64;
    let mut checksum = 0_i64;
    for i in 0..size {
        for j in (i + 1)..size {
            let a = rand_at(i, seed);
            let b = rand_at(j, seed);
            comparisons += 1;
            if a > b {
                checksum += (a - b) as i64;
            } else {
                checksum += (b - a) as i64;
            }
        }
    }
    comparisons + checksum
}

fn main() {
    println!("{}", sort_kernel(2_000, 42));
}
