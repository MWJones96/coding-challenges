use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq)]
pub enum Token {
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Colon,
    Comma,
    StringLiteral(String),
    UnfinishedString(String),
    Number(f64),
    Boolean(bool),
    Null,
    Unknown,
}

pub struct Lexer;
impl Lexer {
    pub fn get_tokens(json_string: &str) -> Vec<Token> {
        let mut tokens = vec![];
        let mut chars = json_string.chars().peekable();
        loop {
            Self::consume_whitespace(&mut chars);

            if let Some(c) = chars.peek() {
                let token = match c {
                    '{' => Self::consume_left_brace(&mut chars),
                    '}' => Self::consume_right_brace(&mut chars),
                    '[' => Self::consume_left_bracket(&mut chars),
                    ']' => Self::consume_right_bracket(&mut chars),
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

        let mut parsed_str = String::new();
        while let Some(c) = chars.next() {
            if c == '"' {
                return Token::StringLiteral(parsed_str);
            }
            if c == '\\' {
                let next_char = chars.next().unwrap();
                match next_char {
                    'n' => parsed_str.push('\n'),
                    'b' => parsed_str.push('\u{0008}'),
                    'f' => parsed_str.push('\u{000C}'),
                    'r' => parsed_str.push('\r'),
                    't' => parsed_str.push('\t'),
                    '"' | '\\' | '/' => parsed_str.push(next_char),
                    _ => parsed_str.push(next_char),
                }
            } else {
                parsed_str.push(c);
            }
        }

        Token::UnfinishedString(parsed_str)
    }

    fn consume_other(chars: &mut Peekable<Chars<'_>>) -> Token {
        let mut keyword: String = String::new();
        while let Some(c) = chars.peek() {
            if c.is_ascii_whitespace()
                || *c == ','
                || *c == ':'
                || *c == '{'
                || *c == '}'
                || *c == '['
                || *c == ']'
                || *c == '"'
            {
                break;
            }
            keyword.push(*c);
            chars.next();
        }
        match keyword.as_str() {
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "null" => Token::Null,
            kwd => {
                let parse_num = kwd.parse::<f64>();
                if let Ok(i) = parse_num {
                    Token::Number(i)
                } else {
                    Token::Unknown
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

    fn consume_left_bracket(chars: &mut Peekable<Chars<'_>>) -> Token {
        chars.next();
        Token::LeftBracket
    }

    fn consume_right_bracket(chars: &mut Peekable<Chars<'_>>) -> Token {
        chars.next();
        Token::RightBracket
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
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
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
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
            Token::Comma,
            Token::StringLiteral(String::from("key2")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
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
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
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
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
            Token::Comma,
            Token::Unknown,
            Token::Colon,
            Token::StringLiteral(String::from("value")),
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
            Token::StringLiteral(String::from("key1")),
            Token::Colon,
            Token::Boolean(true),
            Token::Comma,
            Token::StringLiteral(String::from("key2")),
            Token::Colon,
            Token::Boolean(false),
            Token::Comma,
            Token::StringLiteral(String::from("key3")),
            Token::Colon,
            Token::Null,
            Token::Comma,
            Token::StringLiteral(String::from("key4")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
            Token::Comma,
            Token::StringLiteral(String::from("key5")),
            Token::Colon,
            Token::Number(101.0),
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
            Token::StringLiteral(String::from("key1")),
            Token::Colon,
            Token::Boolean(true),
            Token::Comma,
            Token::StringLiteral(String::from("key2")),
            Token::Colon,
            Token::Unknown,
            Token::Comma,
            Token::StringLiteral(String::from("key3")),
            Token::Colon,
            Token::Null,
            Token::Comma,
            Token::StringLiteral(String::from("key4")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
            Token::Comma,
            Token::StringLiteral(String::from("key5")),
            Token::Colon,
            Token::Number(101.0),
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_with_square_brackets() {
    let test_string = include_str!("../tests/fixtures/step4/valid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
            Token::Comma,
            Token::StringLiteral(String::from("key-n")),
            Token::Colon,
            Token::Number(101.0),
            Token::Comma,
            Token::StringLiteral(String::from("key-o")),
            Token::Colon,
            Token::LeftBrace,
            Token::RightBrace,
            Token::Comma,
            Token::StringLiteral(String::from("key-l")),
            Token::Colon,
            Token::LeftBracket,
            Token::RightBracket,
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_with_inner_object_and_array() {
    let test_string = include_str!("../tests/fixtures/step4/valid2.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
            Token::Comma,
            Token::StringLiteral(String::from("key-n")),
            Token::Colon,
            Token::Number(101.0),
            Token::Comma,
            Token::StringLiteral(String::from("key-o")),
            Token::Colon,
            Token::LeftBrace,
            Token::StringLiteral(String::from("inner key")),
            Token::Colon,
            Token::StringLiteral(String::from("inner value")),
            Token::RightBrace,
            Token::Comma,
            Token::StringLiteral(String::from("key-l")),
            Token::Colon,
            Token::LeftBracket,
            Token::StringLiteral(String::from("list value")),
            Token::RightBracket,
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_with_inner_object_and_array_with_bad_string_literal() {
    let test_string = include_str!("../tests/fixtures/step4/invalid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(String::from("value")),
            Token::Comma,
            Token::StringLiteral(String::from("key-n")),
            Token::Colon,
            Token::Number(101.0),
            Token::Comma,
            Token::StringLiteral(String::from("key-o")),
            Token::Colon,
            Token::LeftBrace,
            Token::StringLiteral(String::from("inner key")),
            Token::Colon,
            Token::StringLiteral(String::from("inner value")),
            Token::RightBrace,
            Token::Comma,
            Token::StringLiteral(String::from("key-l")),
            Token::Colon,
            Token::LeftBracket,
            Token::Unknown,
            Token::Unknown,
            Token::RightBracket,
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_supports_string_with_escape_characters() {
    let test_string = include_str!("../tests/fixtures/step4/valid3.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);
    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(String::from("v\n\u{0008}\u{000C}\r\tal\"ue\"aaaaaa\\/")),
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_supports_various_number_formats() {
    let test_string = include_str!("../tests/fixtures/step4/valid4.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![
            Token::LeftBracket,
            Token::Number(100.0),
            Token::Comma,
            Token::Number(100.5),
            Token::Comma,
            Token::Number(100.0),
            Token::Comma,
            Token::Number(100.0),
            Token::Comma,
            Token::Number(-1.5),
            Token::Comma,
            Token::Number(-1.0),
            Token::RightBracket,
        ],
        tokens
    );
}

#[test]
fn test_lexer_supports_unterminated_string() {
    let test_string = "\"this string is not terminated correctly";
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(
        vec![Token::UnfinishedString(String::from(
            "this string is not terminated correctly"
        ))],
        tokens
    );
}
