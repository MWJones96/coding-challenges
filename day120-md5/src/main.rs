use std::{
    fs::{self},
    io::{self, Read},
};

use clap::{CommandFactory, Parser};

mod process;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(num_args = 0..)]
    files: Vec<String>,

    #[arg(short, long)]
    binary: bool,

    #[arg(short, long)]
    check: bool,
}

fn process_stdin() -> i32 {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer).unwrap();

    let output = process::process(buffer);
    println!("{}  -", output);

    0
}

fn print_file_hash(bytes: Vec<u8>, b: bool, file: &str) {
    let mark = match b {
        true => "*",
        false => " ",
    };
    let output = process::process(bytes);
    println!("{} {}{}", output, mark, file);
}

fn print_file_checks(bytes: Vec<u8>) -> i32 {
    let mut no_matches = 0;
    let mut ret = 0;

    let checksum_lines = std::str::from_utf8(&bytes).unwrap();
    for line in checksum_lines.lines() {
        let line: String = line.replace('*', " ");
        let line: Vec<&str> = line.split_whitespace().collect();
        let precomputed_hash = line[0];
        let file_to_check = line[1];

        match fs::read(file_to_check) {
            Ok(bytes) => {
                let output = process::process(bytes);
                if output == precomputed_hash {
                    println!("{}: OK", &file_to_check);
                } else {
                    println!("{}: FAILED", &file_to_check);
                    no_matches += 1;
                    ret = 1;
                }
            }
            Err(_) => todo!(),
        }
    }
    if no_matches > 0 {
        eprintln!(
            "{}: WARNING: {} computed checksum{} did NOT match",
            Args::command().get_name(),
            no_matches,
            if no_matches > 1 { "s" } else { "" }
        );
    }

    ret
}

fn process_files(files: Vec<String>, b: bool, c: bool) -> i32 {
    let mut ret = 0;
    for file in files {
        if file == "-" {
            process_stdin();
        } else {
            match fs::read(&file) {
                Ok(bytes) => {
                    if c {
                        ret = print_file_checks(bytes);
                    } else {
                        print_file_hash(bytes, b, &file);
                    }
                }
                Err(_) => {
                    eprintln!(
                        "{}: {}: No such file or directory",
                        Args::command().get_name(),
                        &file
                    );
                    ret = 1;
                }
            }
        }
    }

    ret
}

fn main() {
    let args = Args::parse();

    let code = match args.files.is_empty() {
        true => process_stdin(),
        false => process_files(args.files, args.binary, args.check),
    };

    std::process::exit(code);
}
