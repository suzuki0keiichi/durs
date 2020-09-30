use std::env;
use std::fs;

struct Options {
    max_depth: i32,
    threshold: u64,
    human_readable: bool,
}

fn parse_int_arg(prefixes: &[&str], arg: &String) -> Result<Option<u64>, String> {
    for prefix in prefixes.iter() {
        if !arg.starts_with(prefix) {
            continue;
        }

        let result = match arg.get(prefix.len()..arg.len()) {
            Some(value) => {
                match value.parse() {
                    Ok(value) => Ok(Some(value)),
                    Err(_) => Err(format!("{} is not number {}", arg, value)),
                }
            }
            None => Err(format!("{} parameter not found", arg)),
        };

        return result;
    }

    Ok(None)
}

fn parse_args(args: Vec<String>) -> Result<(Vec<String>, Options), String> {
    let mut max_depth: Option<i32> = None;
    let mut threshold: Option<u64> = None;
    let mut human_readable: Option<bool> = None;
    let mut paths: Vec<String> = Vec::new();

    for arg in args.iter().skip(1) {
        if let Some(num) = parse_int_arg(&["-d", "--max-depth="], &arg)? {
            max_depth = Some(num as i32);
        } else if let Some(num) = parse_int_arg(&["-t", "--threshold="], &arg)? {
            threshold = Some(num);
        } else if arg == "-h" || arg == "--human-readable" {
            human_readable = Some(true);
        } else if arg.starts_with("-") {
            println!("unknown option {}", arg);
        } else {
            paths.push(arg.clone());
        }
    }

    if paths.is_empty() {
        paths.push(".".into());
    }

    Ok((paths, Options {
        max_depth: max_depth.unwrap_or(3),
        human_readable: human_readable.unwrap_or(false),
        threshold: threshold.unwrap_or(0),
    }))
}

fn main() {
    match parse_args(env::args().collect()) {
        Ok((paths, options)) => {
            for path in paths.iter() {
                display_dir(path, 0, &options);
            }
        }
        Err(err) => {
            println!("option parse failed. {}", err);
        }
    };
}

fn display_dir(dir_name: &String, depth: i32, options: &Options) -> u64 {
    let mut total: u64 = 0;

    let dir = fs::read_dir(format!("{}", dir_name));
    if dir.is_err() {
        // println!("{} search error {}", dir, dir.err().unwrap());
        return 0;
    }

    for entry in dir.unwrap() {
        let entry = entry.unwrap();

        entry.file_name().to_str().map(|file_name| {
            if entry.file_type().unwrap().is_dir() {
                total += display_dir(&format!("{}/{}", dir_name, file_name), depth + 1, options);
            } else {
                let len = entry.metadata().unwrap().len();

                total += len;
                display_path(&format!("{}/{}", *dir_name, file_name), len, depth + 1, options);
            }
        });
    }

    display_path(dir_name, total, depth, options);

    total
}

fn display_path(name: &String, size: u64, depth: i32, options: &Options) {
    if options.max_depth - depth >= 0 && size > options.threshold {
        println!("{} {}", format(size, options.human_readable), name);
    }
}

fn format(size: u64, human_readable: bool) -> String {
    if !human_readable {
        format!("{: >12}", size)
    } else if size < 1000 {
        format!("{: >5}", size)
    } else if size < 1000000 {
        format!("{: >5}K", size / 1000)
    } else if size < 1000000000 {
        format!("{: >5}M", size / 1000000)
    } else if size < 1000000000000 {
        format!("{: >5}G", size / 1000000000)
    } else {
        format!("{: >5}T", size / 1000000000000)
    }
}