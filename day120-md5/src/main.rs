use std::{
    fs,
    io::{self, Read},
};

use clap::{CommandFactory, Parser};

mod process;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(num_args = 1..)]
    files: Vec<String>,
}

fn print_stdin() {
    let mut buffer = Vec::new();
    io::stdin().read_to_end(&mut buffer).unwrap();

    let output = process::process(buffer);
    println!("{}  -", output);
}

fn main() {
    let args = Args::parse();
    let mut has_error = false;

    if args.files.is_empty() {
        print_stdin();
    } else {
        for file in args.files {
            if file == "-" {
                print_stdin();
            } else {
                match fs::read(&file) {
                    Ok(bytes) => {
                        let output = process::process(bytes);
                        println!("{}  {}", output, &file);
                    }
                    Err(_) => {
                        eprintln!(
                            "{}: {}: No such file or directory",
                            Args::command().get_name(),
                            &file
                        );
                        has_error = true;
                    }
                }
            }
        }
    }

    if has_error {
        std::process::exit(1);
    }
}
