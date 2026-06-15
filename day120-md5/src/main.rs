use std::{
    fs::{self, File},
    io::{self, BufRead, Read},
};

use clap::{CommandFactory, Parser};

mod process;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(num_args = 1..)]
    files: Vec<String>,

    #[arg(short, long)]
    binary: bool,

    #[arg(short, long)]
    check: Option<String>,
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

    if let Some(file) = args.check {
        let file = File::open(file).unwrap();
        let reader = io::BufReader::new(file);
        let mut no_matches = 0;

        for line in reader.lines() {
            let line: String = line.unwrap().replace('*', " ");
            let line: Vec<&str> = line.split_whitespace().collect();
            let hash = line[0];
            let file_h = line[1];

            match fs::read(file_h) {
                Ok(bytes) => {
                    let output = process::process(bytes);
                    if output == hash {
                        println!("{}: OK", &file_h);
                    } else {
                        eprintln!("{}: FAILED", &file_h);
                        no_matches += 1;
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
    } else if args.files.is_empty() {
        print_stdin();
    } else {
        for file in args.files {
            if file == "-" {
                print_stdin();
            } else {
                match fs::read(&file) {
                    Ok(bytes) => {
                        let mark = match args.binary {
                            true => "*",
                            false => " ",
                        };
                        let output = process::process(bytes);
                        println!("{} {}{}", output, mark, &file);
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
