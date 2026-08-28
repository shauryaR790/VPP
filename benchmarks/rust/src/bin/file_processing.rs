use std::fs;
use std::path::PathBuf;

fn file_workload(path: &PathBuf, lines: i32) -> usize {
    let body: String = (0..lines).map(|_| "benchmark-line\n").collect();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(path, &body).expect("write");
    fs::read_to_string(path).expect("read").len()
}

fn main() {
    let path = PathBuf::from("benchmarks/results/tmp/bench-io.txt");
    println!("{}", file_workload(&path, 5000));
}
