use clap::Parser;
use std::fs::File;
use std::io::Read;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'c', value_name = "FILE", help = "Get number of bytes in file")]
    filename_bytes: Vec<String>,

    #[arg(short = 'l', value_name = "FILE", help = "Get number of lines in file")]
    filename_lines: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    for filename_bytes in args.filename_bytes {
        // 1. Open the file
        let mut file = File::open(&filename_bytes)?;
        // 2. Prepare a buffer (String)
        let mut contents = String::new();
        // 3. Read into the buffer
        file.read_to_string(&mut contents)?;
        let contents = contents.as_bytes().to_vec();
        println!("{:8} {}", get_num_bytes(&contents), &filename_bytes);
    }

    for filename_lines in args.filename_lines {
        // 1. Open the file
        let mut file = File::open(&filename_lines)?;
        // 2. Prepare a buffer (String)
        let mut contents = String::new();
        // 3. Read into the buffer
        file.read_to_string(&mut contents)?;
        let contents = contents.as_bytes().to_vec();
        println!("{:8} {}", get_num_lines(&contents), &filename_lines);
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
