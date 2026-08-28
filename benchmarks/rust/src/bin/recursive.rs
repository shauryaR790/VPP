fn recurse(depth: i32) -> i64 {
    if depth == 0 {
        return 1;
    }
    recurse(depth - 1) + recurse(depth - 1)
}

fn main() {
    println!("{}", recurse(20));
}
