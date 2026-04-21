use clap::Parser;
use std::fs::File;
use std::io::Read;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 'c', value_name = "FILE", help = "Get number of bytes in file")]
    filename: String,
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // 1. Open the file
    let mut file = File::open(&args.filename)?;

    // 2. Prepare a buffer (String)
    let mut contents = String::new();
    // 3. Read into the buffer
    file.read_to_string(&mut contents)?;
    let mut contents = contents.as_bytes().to_vec();
    println!("  {} {}", get_num_bytes(&contents), args.filename);

    Ok(())
}

fn get_num_bytes(bytes: &Vec<u8>) -> usize {
    bytes.len()
}

#[test]
fn test_num_bytes() {
    let input = include_bytes!("../test.txt");
    assert_eq!(342_190, get_num_bytes(&input.to_vec()));
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
    assert_eq!("  1 test2.txt\n", stdout);
}
