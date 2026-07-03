use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
    StringLiteral,
    Colon,
    Comma,
    True,
    False,
    Null,
    Number,
    UnknownKeyword,
}

struct Lexer;
impl Lexer {
    fn get_tokens(json_string: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut chars = json_string.chars().peekable();
        loop {
            Self::consume_whitespace(&mut chars);

            if let Some(c) = chars.peek() {
                dbg!(c);
                let token = match c {
                    '{' => Self::consume_left_brace(&mut chars),
                    '}' => Self::consume_right_brace(&mut chars),
                    '"' => Self::consume_string_literal(&mut chars),
                    ':' => Self::consume_colon(&mut chars),
                    ',' => Self::consume_comma(&mut chars),
                    _ => Self::consume_other(&mut chars),
                };
                tokens.push(token);
            } else {
                break;
            }
        }

        tokens
    }

    fn consume_string_literal(chars: &mut Peekable<Chars<'_>>) -> Token {
        assert!(chars.peek().is_some() && *chars.peek().unwrap() == '"');

        //Consume first "
        chars.next();
        while let Some(c) = chars.peek() {
            if *c == '"' {
                break;
            }

            chars.next();
        }

        chars.next();
        Token::StringLiteral
    }

    fn consume_other(chars: &mut Peekable<Chars<'_>>) -> Token {
        let mut keyword: String = String::new();
        while let Some(c) = chars.peek() {
            match *c {
                'a'..='z' | 'A'..='Z' | '0'..='9' => {
                    keyword.push(*c);
                    chars.next();
                }
                _ => {
                    break;
                }
            }
        }
        match keyword.as_str() {
            "true" => Token::True,
            "false" => Token::False,
            "null" => Token::Null,
            kwd => {
                let parse_num = kwd.parse::<u64>();
                match parse_num {
                    Ok(_) => Token::Number,
                    Err(_) => Token::UnknownKeyword,
                }
            }
        }
    }

    fn consume_whitespace(chars: &mut Peekable<Chars<'_>>) {
        while let Some(c) = chars.peek() {
            if c.is_ascii_whitespace() {
                chars.next();
            } else {
                break;
            }
        }
    }

    fn consume_left_brace(chars: &mut Peekable<Chars<'_>>) -> Token {
        chars.next();
        Token::LeftBrace
    }

    fn consume_right_brace(chars: &mut Peekable<Chars<'_>>) -> Token {
        chars.next();
        Token::RightBrace
    }

    fn consume_comma(chars: &mut Peekable<Chars<'_>>) -> Token {
        chars.next();
        Token::Comma
    }

    fn consume_colon(chars: &mut Peekable<Chars<'_>>) -> Token {
        chars.next();
        Token::Colon
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
            Token::Colon,
            Token::StringLiteral,
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_keywords_and_numbers() {
    let test_string = include_str!("../tests/fixtures/step3/valid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral,
            Token::Colon,
            Token::True,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::False,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::Null,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::StringLiteral,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::Number,
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_badly_formed_keyword() {
    let test_string = include_str!("../tests/fixtures/step3/invalid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral,
            Token::Colon,
            Token::True,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::UnknownKeyword,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::Null,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::StringLiteral,
            Token::Comma,
            Token::StringLiteral,
            Token::Colon,
            Token::Number,
            Token::RightBrace
        ],
        tokens
    );
}
