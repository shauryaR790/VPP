fn fib(n: i32) -> i64 {
    if n <= 1 {
        return n as i64;
    }
    fib(n - 1) + fib(n - 2)
}

fn main() {
    println!("{}", fib(35));
}
