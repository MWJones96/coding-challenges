use std::{
    fs::{self},
    io::{self, Read},
};

use clap::{CommandFactory, Parser};
use regex::Regex;

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

fn print_file_checks(bytes: Vec<u8>, file: &str) -> i32 {
    let mut no_matches = 0;
    let mut bad_lines = 0;
    let mut bad_file_paths: Vec<String> = vec![];
    let mut ret = 0;
    let re = Regex::new(r"^([0-9a-f]+)(?:  | \*)(.+)+$").unwrap();

    let checksum_lines = std::str::from_utf8(&bytes).unwrap();
    let cs_line_count = checksum_lines.lines().count();
    for line in checksum_lines.lines() {
        let caps = re.captures(line.trim());
        if caps.is_none() {
            bad_lines += 1;
            continue;
        }
        let caps = caps.unwrap();

        let precomputed_hash = &caps[1];
        let file_to_check = &caps[2];

        match fs::read(file_to_check) {
            Ok(bytes) => {
                let output = process::process(bytes);
                if output == precomputed_hash {
                    println!("{}: OK", &file_to_check);
                } else {
                    println!("{}: FAILED", &file_to_check);
                    no_matches += 1;
                }
            }
            Err(_) => {
                println!("{}: FAILED open or read", &file_to_check);
                bad_file_paths.push(file_to_check.to_string());
            }
        }
    }

    if no_matches > 0 {
        eprintln!(
            "{}: WARNING: {} computed checksum{} did NOT match",
            Args::command().get_name(),
            no_matches,
            if no_matches > 1 { "s" } else { "" }
        );

        ret = 1;
    }

    if bad_lines == cs_line_count {
        eprintln!(
            "{}: {}: no properly formatted checksum lines found",
            Args::command().get_name(),
            file,
        );
    } else if bad_lines > 0 {
        eprintln!(
            "{}: WARNING: {} line{} improperly formatted",
            Args::command().get_name(),
            bad_lines,
            if bad_lines > 1 { "s are" } else { " is" }
        );
    }

    if bad_file_paths.len() > 0 {
        for file in bad_file_paths.iter() {
            eprintln!(
                "{}: {}: No such file or directory",
                Args::command().get_name(),
                &file
            );
        }

        eprintln!(
            "{}: WARNING: {} listed file{} could not be read",
            Args::command().get_name(),
            bad_file_paths.len(),
            if bad_file_paths.len() > 1 { "s" } else { "" }
        );

        ret = 1;
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
                        if print_file_checks(bytes, &file) > 0 {
                            ret = 1;
                        }
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
