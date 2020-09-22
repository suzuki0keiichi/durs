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

fn parse_options(args: Vec<String>) -> Result<Options, String> {
    let mut max_depth: Option<i32> = None;
    let mut threshold: Option<u64> = None;
    let mut human_readable: Option<bool> = None;

    for arg in args.iter() {
        if let Some(num) = parse_int_arg(&["-d", "--max-depth="], &arg)? {
            max_depth = Some(num as i32);
        }

        if let Some(num) = parse_int_arg(&["-t", "--threshold="], &arg)? {
            threshold = Some(num);
        }

        if arg == "-h" || arg == "--human-readable" {
            human_readable = Some(true);
        }
    }

    Ok(Options {
        max_depth: max_depth.unwrap_or(3),
        human_readable: human_readable.unwrap_or(false),
        threshold: threshold.unwrap_or(0),
    })
}

fn main() {
    match parse_options(env::args().collect()) {
        Ok(options) => {
            calc(String::from("."), 0, &options)
        }
        Err(err) => {
            println!("option parse failed. {}", err);
            0 // matchで型を揃えないと死んでもコンパイルしないみたいなことになってるから仕方なくやっているが不毛に思う
        }
    };
}

/// calcだけど表示もしてしまっている、、メモリ無視なら結果は別で保存したほうが良いか？
fn calc(dir_name: String, depth: i32, options: &Options) -> u64 {
    let mut total: u64 = 0;

    let dir = fs::read_dir(format!("{}", dir_name));
    if dir.is_err() {
        // println!("{} search error {}", dir, dir.err().unwrap());
        return 0;
    }

    for entry in dir.unwrap() {
        let entry = entry.unwrap();

        if entry.file_type().unwrap().is_dir() {
            total += calc(format!("{}/{}", dir_name, entry.file_name().to_str().unwrap()), depth + 1, options);
        } else {
            let len = entry.metadata().unwrap().len();

            total += len;
            display(format!("{}/{}", dir_name, entry.file_name().to_str().unwrap()), len, depth + 1, options);
        }
    }

    display(dir_name, total, depth, options);

    total
}

fn display(name: String, size: u64, depth: i32, options: &Options) {
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