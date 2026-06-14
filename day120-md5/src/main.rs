use std::{
    fs,
    io::{self, Read},
};

use clap::Parser;

mod process;

#[derive(Parser, Debug)]
struct Args {
    #[arg(num_args = 1..)]
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    if args.files.is_empty() {
        let mut buffer = Vec::new();
        io::stdin().read_to_end(&mut buffer).unwrap();

        let output = process::process(buffer);
        println!("{}  -", output);
    } else {
        for file in args.files {
            let output = process::process(fs::read(&file).unwrap());
            println!("{}  {}", output, &file);
        }
    }
}
