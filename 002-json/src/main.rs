use crate::lexer::Lexer;

mod lexer;
mod parser;

fn main() {
    let i = 0;
    Lexer::get_tokens("");
    println!("Hello, world!");
}
