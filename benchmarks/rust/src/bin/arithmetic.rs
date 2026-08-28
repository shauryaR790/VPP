fn arithmetic(iters: i64) -> i64 {
    let mut x = 1_i64;
    for _ in 0..iters {
        x = x + (x * 3) % 1_000_003;
        if x < 0 {
            x = -x;
        }
        x -= x / 4;
    }
    x
}

fn main() {
    println!("{}", arithmetic(500_000_000));
}
