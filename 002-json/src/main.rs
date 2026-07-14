use crate::lexer::Lexer;

mod lexer;
mod parser;

fn main() {
    let tokens = Lexer::get_tokens("{true}");
    match tokens {
        Ok(_) => println!("Success"),
        Err(_) => println!("Failure"),
    }
    println!("Hello, world!");
}
