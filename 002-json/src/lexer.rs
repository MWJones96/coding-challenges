use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
    StringLiteral,
    Colon,
    Comma,
    UnknownKeyword,
}

struct Lexer;
impl Lexer {
    fn get_tokens(json_string: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut chars = json_string.chars().peekable();
        while let Some(c) = chars.peek() {
            match c {
                '{' => {
                    tokens.push(Token::LeftBrace);
                }
                '}' => {
                    tokens.push(Token::RightBrace);
                }
                '"' => {
                    Self::consume_string_literal(&mut chars);
                    tokens.push(Token::StringLiteral);
                }
                ':' => {
                    tokens.push(Token::Colon);
                }
                ',' => {
                    tokens.push(Token::Comma);
                }
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    Self::consume_keyword(&mut chars);
                    tokens.push(Token::UnknownKeyword);
                }
                _ => {
                    dbg!(c);
                }
            }

            chars.next();
        }

        tokens
    }

    fn consume_string_literal(chars: &mut Peekable<Chars>) {
        assert!(chars.peek().is_some() && *chars.peek().unwrap() == '"');

        //Consume first "
        chars.next();
        while let Some(c) = chars.peek() {
            if *c == '"' {
                break;
            }

            chars.next();
        }
    }

    fn consume_keyword(chars: &mut Peekable<Chars<'_>>) {
        while let Some(c) = chars.peek() {
            match *c {
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    chars.next();
                }
                _ => {
                    break;
                }
            }
        }
    }
}

#[test]
fn test_lexer_empty_object() {
    let test_string = include_str!("../tests/fixtures/step1/valid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(vec![Token::LeftBrace, Token::RightBrace], tokens);
}

#[test]
fn test_lexer_empty_file() {
    let test_string = include_str!("../tests/fixtures/step1/invalid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(Vec::<Token>::new(), tokens);
}

#[test]
fn test_lexer_empty_object_with_whitespace() {
    let test_string = "    {   }   ";
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(vec![Token::LeftBrace, Token::RightBrace], tokens);
}

#[test]
fn test_lexer_key_value_pair() {
    let test_string = include_str!("../tests/fixtures/step2/valid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral,
            Token::Colon,
            Token::StringLiteral,
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_multiple_key_value() {
    let test_string = include_str!("../tests/fixtures/step2/valid2.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral,
            Token::Colon,
            Token::StringLiteral,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::StringLiteral,
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_comma_after_last_key_value() {
    let test_string = include_str!("../tests/fixtures/step2/invalid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral,
            Token::Colon,
            Token::StringLiteral,
            Token::Comma,
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_unknown_keyword() {
    let test_string = include_str!("../tests/fixtures/step2/invalid2.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral,
            Token::Colon,
            Token::StringLiteral,
            Token::Comma,
            Token::UnknownKeyword,
            Token::StringLiteral,
            Token::RightBrace
        ],
        tokens
    );
}
