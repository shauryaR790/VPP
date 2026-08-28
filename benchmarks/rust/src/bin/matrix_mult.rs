fn mat_value(row: i32, col: i32, _n: i32) -> i32 {
    (row * 131 + col * 17) % 997
}

fn matrix_multiply(n: i32) -> i64 {
    let mut checksum = 0_i64;
    for i in 0..n {
        for j in 0..n {
            let mut total = 0_i64;
            for k in 0..n {
                total += mat_value(i, k, n) as i64 * mat_value(k, j, n) as i64;
            }
            checksum += total;
        }
    }
    checksum
}

fn main() {
    println!("{}", matrix_multiply(128));
}
