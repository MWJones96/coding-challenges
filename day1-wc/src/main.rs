use clap::Parser;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Count bytes
    #[arg(short = 'c', long)]
    bytes: bool,

    /// Count lines
    #[arg(short = 'l', long)]
    lines: bool,

    /// Count words
    #[arg(short = 'w', long)]
    words: bool,

    /// Count locale chars
    #[arg(short = 'm', long)]
    chars: bool,

    /// The files to process
    #[arg(value_name = "FILES")]
    files: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    for filename in args.files {
        // 1. Open the file
        let mut file = File::open(&filename)?;
        // 2. Prepare a buffer (String)
        let mut contents = String::new();
        // 3. Read into the buffer
        file.read_to_string(&mut contents)?;
        let contents = contents.as_bytes().to_vec();
        if args.bytes {
            println!("{:8} {}", get_num_bytes(&contents), &filename);
        }
        if args.lines {
            println!("{:8} {}", get_num_lines(&contents), &filename);
        }
        if args.words {
            println!("{:8} {}", get_num_words(&contents), &filename);
        }
        if args.chars {
            println!("{:8} {}", get_num_chars_locale(&contents), &filename);
        }
    }

    Ok(())
}

fn get_num_bytes(bytes: &Vec<u8>) -> usize {
    bytes.len()
}

fn get_num_lines(bytes: &Vec<u8>) -> usize {
    let mut count = 0;
    for byte in bytes {
        if *byte == 0x0a {
            count += 1;
        }
    }

    return count;
}

fn get_num_words(bytes: &Vec<u8>) -> usize {
    let re = Regex::new(r"\S+").unwrap();
    let text = str::from_utf8(bytes).expect("Invalid UTF8");
    let matches: Vec<_> = re.find_iter(text).into_iter().collect();

    matches.len()
}

fn get_num_chars_locale(bytes: &Vec<u8>) -> usize {
    let text = str::from_utf8(bytes).expect("Invalid UTF8");
    text.chars().count()
}

#[test]
fn test_num_bytes() {
    let input = include_bytes!("../test.txt");
    assert_eq!(342_190, get_num_bytes(&input.to_vec()));
}

#[test]
fn test_num_lines() {
    let input = include_bytes!("../test.txt");
    assert_eq!(7_145, get_num_lines(&input.to_vec()));
}

#[test]
fn test_num_words() {
    let input = include_bytes!("../test.txt");
    assert_eq!(58_164, get_num_words(&input.to_vec()));
}

#[test]
fn test_num_chars_locale() {
    let input = include_bytes!("../test.txt");
    assert_eq!(339_292, get_num_chars_locale(&input.to_vec()));
}

#[test]
fn test_output_num_bytes() {
    let output = Command::new("cargo")
        .args(["run", "--", "-c", "test.txt"])
        .output()
        .expect("Failed to execute");

    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF8");
    assert_eq!("  342190 test.txt\n", stdout);
}

#[test]
fn test_output_num_bytes_test2() {
    let output = Command::new("cargo")
        .args(["run", "--", "-c", "test2.txt"])
        .output()
        .expect("Failed to execute");

    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF8");
    assert_eq!("       1 test2.txt\n", stdout);
}

#[test]
fn test_output_num_lines() {
    let output = Command::new("cargo")
        .args(["run", "--", "-l", "test.txt"])
        .output()
        .expect("Failed to execute");

    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF8");
    assert_eq!("    7145 test.txt\n", stdout);
}

#[test]
fn test_output_num_lines_test2() {
    let output = Command::new("cargo")
        .args(["run", "--", "-l", "test2.txt"])
        .output()
        .expect("Failed to execute");

    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF8");
    assert_eq!("       0 test2.txt\n", stdout);
}

#[test]
fn test_output_num_words() {
    let output = Command::new("cargo")
        .args(["run", "--", "-w", "test.txt"])
        .output()
        .expect("Failed to execute");

    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF8");
    assert_eq!("   58164 test.txt\n", stdout);
}

#[test]
fn test_output_num_words_test2() {
    let output = Command::new("cargo")
        .args(["run", "--", "-w", "test2.txt"])
        .output()
        .expect("Failed to execute");

    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF8");
    assert_eq!("       1 test2.txt\n", stdout);
}

#[test]
fn test_output_num_chars_locale() {
    let output = Command::new("cargo")
        .args(["run", "--", "-m", "test.txt"])
        .output()
        .expect("Failed to execute");

    let stdout = str::from_utf8(&output.stdout).expect("Invalid UTF8");
    assert_eq!("  339292 test.txt\n", stdout);
}
