use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut depth: i32 = 3;
    let mut threshold: u64 = 3;
    let mut human_readable: bool = false;

    for arg in args.iter() {
        if arg.starts_with("-d") {
            depth = arg.get(2..arg.len()).unwrap().parse().unwrap();
        }

        if arg.starts_with("--max-depth=") {
            depth = arg.get(12..arg.len()).unwrap().parse().unwrap();
        }

        if arg.starts_with("-t") {
            threshold = arg.get(2..arg.len()).unwrap().parse().unwrap();
        }

        if arg.starts_with("--threshold=") {
            threshold = arg.get(12..arg.len()).unwrap().parse().unwrap();
        }

        if arg == "-h" || arg == "--human-readable" {
            human_readable = true;
        }
    }

    calc(String::from("."), depth, threshold, human_readable);
}

/// calcだけど表示もしてしまっている、、メモリ無視なら結果は別で保存したほうが良いか？
fn calc(dir: String, depth: i32, threshold: u64, human_readable: bool) -> u64 {
    let mut total: u64 = 0;

    let readdir = fs::read_dir(format!("{}", dir));
    if readdir.is_err() {
        // println!("{} search error {}", dir, readdir.err().unwrap());
        return 0;
    }

    for entry in readdir.unwrap() {
        let entry = entry.unwrap();

        if entry.file_type().unwrap().is_dir() {
            total += calc(format!("{}/{}", dir, entry.file_name().to_str().unwrap()), depth - 1, threshold, human_readable);
        } else {
            let len = entry.metadata().unwrap().len();
            total += len;
            if depth > 0 && len > threshold {
                println!("{0: >5} {1:}", format(len, human_readable), format!("{}/{}", dir, entry.file_name().to_str().unwrap()));
            }
        }
    }

    if depth >= 0 && total > threshold {
        println!("{0: >5} {1:}", format(total, human_readable), dir);
    }

    total
}

fn format(size: u64, human_readable: bool) -> String {
    if !human_readable {
        format!("{: >12}", size)
    } else if size < 1000 {
        format!("{}", size)
    } else if size < 1000000 {
        format!("{}k", size / 1000)
    } else if size < 1000000000 {
        format!("{}M", size / 1000000)
    } else if size < 1000000000000 {
        format!("{}G", size / 1000000000)
    } else {
        format!("{}T", size / 1000000000000)
    }
}