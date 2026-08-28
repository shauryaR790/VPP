fn sum_range(n: i64) -> i64 {
    let mut total = 0_i64;
    for i in 0..n {
        total += i;
    }
    total
}

fn main() {
    println!("{}", sum_range(5_000_000));
}
