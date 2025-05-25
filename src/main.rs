use rayon::prelude::*;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

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

    for arg in args.into_iter().skip(1) {
        if let Some(num) = parse_int_arg(&["-d", "--max-depth="], &arg)? {
            max_depth = Some(num as i32);
        } else if let Some(num) = parse_int_arg(&["-t", "--threshold="], &arg)? {
            threshold = Some(num);
        } else if arg == "-h" || arg == "--human-readable" {
            human_readable = true;
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: {arg}"));
        } else {
            paths.push(arg);
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
    let result: Result<(), String> = (|| {
        let (paths, options) = parse_args(env::args().collect())?;
        
        // 出力バッファを作成
        let output_buffer = Mutex::new(Vec::new());
        
        for path in paths {
            let path_buf = PathBuf::from(path);
            display_dir(&path_buf, 0, &options, &output_buffer);
        }
        
        // バッファの内容を一度に出力
        let buffer = output_buffer.lock().unwrap();
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(&buffer).unwrap();
        
        Ok(())
    })();

    if let Err(err) = result {
        eprintln!("Failed to parse options: {err}");
    }
}

fn display_dir(dir: &Path, depth: i32, options: &Options, output_buffer: &Mutex<Vec<u8>>) -> u64 {
    let total = AtomicU64::new(0);

    let mut entries: Vec<_> = match fs::read_dir(dir) {
        Ok(entries) => entries.filter_map(Result::ok).collect(),
        Err(e) => {
            eprintln!("Failed to read {}: {e}", dir.display());
            return 0;
        }
    };

    // ディレクトリを先に処理するためにソート（メモリ効率向上）
    entries.sort_by_key(|entry| {
        entry.file_type()
            .map(|ft| !ft.is_dir())
            .unwrap_or(true)
    });

    // 並列処理でエントリを処理
    entries.par_iter().for_each(|entry| {
        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(e) => {
                eprintln!("Failed to get file type for {}: {e}", entry.path().display());
                return;
            }
        };

        let path = entry.path();
        if file_type.is_dir() {
            let size = display_dir(&path, depth + 1, options, output_buffer);
            total.fetch_add(size, Ordering::Relaxed);
        } else {
            // ファイルの場合のみメタデータを取得
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Failed to get metadata for {}: {e}", path.display());
                    return;
                }
            };
            let len = metadata.len();
            total.fetch_add(len, Ordering::Relaxed);
            display_path(&path, len, depth + 1, options, output_buffer);
        }
    });

    let total_size = total.load(Ordering::Relaxed);
    display_path(dir, total_size, depth, options, output_buffer);
    total_size
}

fn display_path(path: &Path, size: u64, depth: i32, options: &Options, output_buffer: &Mutex<Vec<u8>>) {
    if options.max_depth - depth >= 0 && size > options.threshold {
        let output = format!(
            "{} {}\n",
            format_size(size, options.human_readable),
            path.display()
        );
        
        let mut buffer = output_buffer.lock().unwrap();
        buffer.extend_from_slice(output.as_bytes());
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
