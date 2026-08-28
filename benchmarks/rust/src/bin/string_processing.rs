fn build_string(iters: i32, chunk: &str) -> usize {
    let mut out = String::new();
    for _ in 0..iters {
        out.push_str(chunk);
    }
    out.len()
}

fn main() {
    println!("{}", build_string(3000, "benchmark-chunk-0123456789"));
}
