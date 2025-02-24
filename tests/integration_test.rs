use std::fs::{self, File};
use std::process::Command;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_basic_directory_scan() {
    // テスト用の一時ディレクトリを作成
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // テスト用のファイル構造を作成
    let test_files = [
        ("file1.txt", 1000),
        ("dir1/file2.txt", 2000),
        ("dir1/dir2/file3.txt", 3000),
    ];

    for (path, size) in &test_files {
        let full_path = temp_path.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(full_path).unwrap();
        file.write_all(&vec![0; *size]).unwrap();
    }

    // dursを実行
    let output = Command::new(env!("CARGO_BIN_EXE_durs"))
        .arg(temp_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    
    // 出力の検証
    assert!(output.status.success());
    assert!(stdout.contains("6000")); // 合計サイズ
    assert!(stdout.contains("1000")); // file1.txt
    assert!(stdout.contains("2000")); // file2.txt
    assert!(stdout.contains("3000")); // file3.txt
}

#[test]
fn test_human_readable_option() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 1MB のファイルを作成
    let file_path = temp_path.join("large_file.txt");
    let mut file = File::create(file_path).unwrap();
    file.write_all(&vec![0; 1_000_000]).unwrap();

    // human-readable オプションでdursを実行
    let output = Command::new(env!("CARGO_BIN_EXE_durs"))
        .arg("--human-readable")
        .arg(temp_path)
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    
    assert!(output.status.success());
    assert!(stdout.contains("    1M")); // human-readable形式で1MB
}

#[test]
fn test_max_depth_option() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 3階層のディレクトリ構造を作成
    let test_files = [
        ("file1.txt", 1000),
        ("dir1/file2.txt", 1000),
        ("dir1/dir2/file3.txt", 1000),
    ];

    for (path, size) in &test_files {
        let full_path = temp_path.join(path);
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut file = File::create(full_path).unwrap();
        file.write_all(&vec![0; *size]).unwrap();
    }

    // max-depth=1 でdursを実行
    let output = Command::new(env!("CARGO_BIN_EXE_durs"))
        .args(["--max-depth=1", temp_path.to_str().unwrap()])
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    
    assert!(output.status.success());
    // depth <= 1 の要素が表示される：
    // - file1.txt (depth=1)
    // - dir1 (depth=1)
    // - ルートディレクトリ (depth=0)
    assert_eq!(stdout.lines().count(), 3);
}