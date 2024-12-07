use std::env;
use std::fs;
use std::path::{Path, PathBuf};

struct Options {
    max_depth: i32,
    threshold: u64,
    human_readable: bool,
}

fn parse_int_arg(prefixes: &[&str], arg: &str) -> Result<Option<u64>, String> {
    for prefix in prefixes {
        if let Some(value_str) = arg.strip_prefix(prefix) {
            let value = value_str
                .parse::<u64>()
                .map_err(|_| format!("'{}' の値部分 '{}' は数値ではありません", arg, value_str))?;
            return Ok(Some(value));
        }
    }
    Ok(None)
}

fn parse_args(args: Vec<String>) -> Result<(Vec<String>, Options), String> {
    let mut max_depth: Option<i32> = None;
    let mut threshold: Option<u64> = None;
    let mut human_readable: bool = false;
    let mut paths: Vec<String> = Vec::new();

    for arg in args.iter().skip(1) {
        if let Some(num) = parse_int_arg(&["-d", "--max-depth="], arg)? {
            max_depth = Some(num as i32);
        } else if let Some(num) = parse_int_arg(&["-t", "--threshold="], arg)? {
            threshold = Some(num);
        } else if arg == "-h" || arg == "--human-readable" {
            human_readable = true;
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: {}", arg));
        } else {
            paths.push(arg.to_string());
        }
    }

    if paths.is_empty() {
        paths.push(".".into());
    }

    Ok((
        paths,
        Options {
            max_depth: max_depth.unwrap_or(3),
            human_readable,
            threshold: threshold.unwrap_or(0),
        },
    ))
}

fn main() {
    match parse_args(env::args().collect()) {
        Ok((paths, options)) => {
            for path in &paths {
                let path_buf = PathBuf::from(path);
                display_dir(&path_buf, 0, &options);
            }
        }
        Err(err) => eprintln!("Failed to parse options: {}", err),
    }
}

fn display_dir(dir: &Path, depth: i32, options: &Options) -> u64 {
    let mut total: u64 = 0;

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Failed to read {}: {}", dir.display(), e);
            return 0;
        }
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                eprintln!("Failed to get metadata for {}: {}", entry.path().display(), e);
                continue;
            }
        };

        let path = entry.path();
        if file_type.is_dir() {
            total += display_dir(&path, depth + 1, options);
        } else {
            let len = match entry.metadata() {
                Ok(m) => m.len(),
                Err(e) => {
                    eprintln!("Failed to get metadata for {}: {}", path.display(), e);
                    continue;
                }
            };

            total += len;
            display_path(&path, len, depth + 1, options);
        }
    }

    display_path(dir, total, depth, options);
    total
}

fn display_path(path: &Path, size: u64, depth: i32, options: &Options) {
    if options.max_depth - depth >= 0 && size > options.threshold {
        println!(
            "{} {}",
            format_size(size, options.human_readable),
            path.display()
        );
    }
}

fn format_size(size: u64, human_readable: bool) -> String {
    if !human_readable {
        format!("{: >12}", size)
    } else if size < 1_000 {
        format!("{: >6}", size)
    } else if size < 1_000_000 {
        format!("{: >5}K", size / 1_000)
    } else if size < 1_000_000_000 {
        format!("{: >5}M", size / 1_000_000)
    } else if size < 1_000_000_000_000 {
        format!("{: >5}G", size / 1_000_000_000)
    } else {
        format!("{: >5}T", size / 1_000_000_000_000)
    }
}
