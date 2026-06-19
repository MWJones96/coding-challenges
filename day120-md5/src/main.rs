use std::{
    fs::{self},
    io::{self, Read},
};

use clap::{CommandFactory, Parser, ValueEnum};
use regex::Regex;

use crate::process::{ComputeHash, md5::MD5, sha1::SHA1, sha256::SHA256, sha512::SHA512};

mod process;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Algorithm {
    MD5,
    SHA256,
    SHA1,
    SHA512,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(num_args = 0..)]
    files: Vec<String>,

    #[arg(short, long)]
    binary: bool,

    #[arg(short, long)]
    check: bool,

    #[arg(long)]
    quiet: bool,

    #[arg(long)]
    status: bool,

    #[arg(short, long, value_enum, default_value_t = Algorithm::MD5)]
    algorithm: Algorithm,
}

fn process_stdin(args: &Args) -> i32 {
    let mut buffer = Vec::new();
    let mut ret = 0;
    io::stdin().read_to_end(&mut buffer).unwrap();

    if args.check {
        let code = print_file_checks(buffer, "-", args);
        ret = ret.max(code);
    } else {
        let output = match args.algorithm {
            Algorithm::MD5 => MD5::process(buffer),
            Algorithm::SHA256 => SHA256::process(buffer),
            Algorithm::SHA1 => SHA1::process(buffer),
            Algorithm::SHA512 => SHA512::process(buffer),
        };

        if !args.status {
            println!("{}  -", output);
        }
    }

    ret
}

fn print_file_hash(bytes: Vec<u8>, file: &str, args: &Args) -> i32 {
    let mark = match args.binary {
        true => "*",
        false => " ",
    };
    let output = match args.algorithm {
        Algorithm::MD5 => MD5::process(bytes),
        Algorithm::SHA256 => SHA256::process(bytes),
        Algorithm::SHA1 => SHA1::process(bytes),
        Algorithm::SHA512 => SHA512::process(bytes),
    };

    if !args.status {
        println!("{} {}{}", output, mark, file);
    }

    0
}

fn print_file_checks(bytes: Vec<u8>, file: &str, args: &Args) -> i32 {
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
                let output = match args.algorithm {
                    Algorithm::MD5 => MD5::process(bytes),
                    Algorithm::SHA256 => SHA256::process(bytes),
                    Algorithm::SHA1 => SHA1::process(bytes),
                    Algorithm::SHA512 => SHA512::process(bytes),
                };

                if output == precomputed_hash {
                    if !args.quiet && !args.status {
                        println!("{}: OK", &file_to_check);
                    }
                } else {
                    if !args.status {
                        println!("{}: FAILED", &file_to_check);
                    }
                    no_matches += 1;
                }
            }
            Err(_) => {
                if !args.status {
                    println!("{}: FAILED open or read", &file_to_check);
                }
                bad_file_paths.push(file_to_check.to_string());
            }
        }
    }

    if no_matches > 0 {
        if !args.status {
            eprintln!(
                "{}: WARNING: {} computed checksum{} did NOT match",
                Args::command().get_name(),
                no_matches,
                if no_matches > 1 { "s" } else { "" }
            );
        }

        ret = ret.max(1);
    }

    if bad_lines == cs_line_count {
        if !args.status {
            eprintln!(
                "{}: {}: no properly formatted checksum lines found",
                Args::command().get_name(),
                file,
            );
        }
    } else if bad_lines > 0 && !args.status {
        eprintln!(
            "{}: WARNING: {} line{} improperly formatted",
            Args::command().get_name(),
            bad_lines,
            if bad_lines > 1 { "s are" } else { " is" }
        );
    }

    if !bad_file_paths.is_empty() {
        for file in bad_file_paths.iter() {
            if !args.status {
                eprintln!(
                    "{}: {}: No such file or directory",
                    Args::command().get_name(),
                    &file
                );
            }
        }

        if !args.status {
            eprintln!(
                "{}: WARNING: {} listed file{} could not be read",
                Args::command().get_name(),
                bad_file_paths.len(),
                if bad_file_paths.len() > 1 { "s" } else { "" }
            );
        }

        ret = ret.max(1);
    }

    ret
}

fn process_files(args: &Args) -> i32 {
    let mut ret = 0;
    for file in &args.files {
        let code = match file.as_str() {
            "-" => process_stdin(args),
            _ => match fs::read(file) {
                Ok(bytes) => {
                    if args.check {
                        print_file_checks(bytes, file, args)
                    } else {
                        print_file_hash(bytes, file, args)
                    }
                }
                Err(_) => {
                    if !args.status {
                        eprintln!(
                            "{}: {}: No such file or directory",
                            Args::command().get_name(),
                            &file
                        );
                    }

                    1
                }
            },
        };
        ret = ret.max(code);
    }

    ret
}

fn main() {
    let mut args = Args::parse();
    if args.files.is_empty() {
        args.files.push("-".to_string());
    }

    std::process::exit(process_files(&args));
}
