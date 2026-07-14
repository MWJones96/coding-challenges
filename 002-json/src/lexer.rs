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
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexErrorType {
    UnexpectedCharacter(char),
    BadNumberFormat(String),
    BadEscapeCharacter(char),
    UnexpectedEOF,
    BadHexString(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct LexError {
    err_type: LexErrorType,
    row: usize,
    col: usize,
}

pub struct Lexer;
impl Lexer {
    pub fn get_tokens(json_string: &str) -> Result<Vec<Token>, LexError> {
        let mut tokens = vec![];
        let mut chars = json_string.chars().peekable();
        while chars.peek().is_some() {
            Self::consume_whitespace(&mut chars);

            if let Some(c) = chars.peek() {
                let token: Result<Token, LexError> = match c {
                    '{' => {
                        chars.next();
                        Ok(Token::LeftBrace)
                    }
                    '}' => {
                        chars.next();
                        Ok(Token::RightBrace)
                    }
                    '[' => {
                        chars.next();
                        Ok(Token::LeftBracket)
                    }
                    ']' => {
                        chars.next();
                        Ok(Token::RightBracket)
                    }
                    '"' => Self::consume_string_literal(&mut chars),
                    ':' => {
                        chars.next();
                        Ok(Token::Colon)
                    }
                    ',' => {
                        chars.next();
                        Ok(Token::Comma)
                    }
                    't' => Self::consume_true(&mut chars),
                    'f' => Self::consume_false(&mut chars),
                    'n' => Self::consume_null(&mut chars),
                    '-' | '1'..='9' => Self::consume_number(&mut chars),
                    other => {
                        return Err(LexError {
                            err_type: LexErrorType::UnexpectedCharacter(*other),
                            row: 0,
                            col: 0,
                        });
                    }
                };

                match token {
                    Ok(t) => tokens.push(t),
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(tokens)
    }

    fn consume_string_literal(chars: &mut Peekable<Chars<'_>>) -> Result<Token, LexError> {
        //Consume first "
        chars.next();

        let mut parsed_str = String::new();
        while let Some(c) = chars.next() {
            if c == '"' {
                return Ok(Token::StringLiteral(parsed_str));
            }
            if c == '\\' {
                if chars.peek().is_none() {
                    return Err(LexError {
                        err_type: LexErrorType::UnexpectedEOF,
                        row: 0,
                        col: 0,
                    });
                }

                let next_char = chars.next().unwrap();
                match next_char {
                    'n' => parsed_str.push('\n'),
                    'b' => parsed_str.push('\u{0008}'),
                    'f' => parsed_str.push('\u{000C}'),
                    'r' => parsed_str.push('\r'),
                    't' => parsed_str.push('\t'),
                    'u' => {
                        let hex_chars: Vec<char> = chars.take(4).collect();
                        if hex_chars.len() < 4 {
                            return Err(LexError {
                                err_type: LexErrorType::UnexpectedEOF,
                                row: 0,
                                col: 0,
                            });
                        }

                        let hex_str = format!(
                            "{}{}{}{}",
                            hex_chars[0], hex_chars[1], hex_chars[2], hex_chars[3]
                        );
                        match hex::decode(&hex_str) {
                            Ok(bytes) => {
                                for byte in bytes {
                                    parsed_str.push(byte as char);
                                }
                            }
                            Err(_) => {
                                return Err(LexError {
                                    err_type: LexErrorType::BadHexString(hex_str),
                                    row: 0,
                                    col: 0,
                                });
                            }
                        }
                    }
                    '"' | '\\' | '/' => parsed_str.push(next_char),
                    c => {
                        return Err(LexError {
                            err_type: LexErrorType::BadEscapeCharacter(c),
                            row: 0,
                            col: 0,
                        });
                    }
                }
            } else {
                parsed_str.push(c);
            }
        }

        Err(LexError {
            err_type: LexErrorType::UnexpectedEOF,
            row: 0,
            col: 0,
        })
    }

    fn consume_number(chars: &mut Peekable<Chars<'_>>) -> Result<Token, LexError> {
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

        let parse_num = keyword.parse::<f64>();
        parse_num.map_or(
            Err(LexError {
                err_type: LexErrorType::BadNumberFormat(keyword),
                row: 0,
                col: 0,
            }),
            |i| Ok(Token::Number(i)),
        )
    }

    fn consume_whitespace(chars: &mut Peekable<Chars<'_>>) {
        while let Some(c) = chars.peek()
            && c.is_ascii_whitespace()
        {
            chars.next();
        }
    }

    fn consume_true(chars: &mut Peekable<Chars<'_>>) -> Result<Token, LexError> {
        let parsed: Vec<char> = chars.take(4).collect();
        if parsed.len() < 4 {
            return Err(LexError {
                err_type: LexErrorType::UnexpectedEOF,
                row: 0,
                col: 0,
            });
        }

        let compare: Vec<char> = vec!['t', 'r', 'u', 'e'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexError {
                    err_type: LexErrorType::UnexpectedCharacter(parsed[i]),
                    row: 0,
                    col: 0,
                });
            }
        }

        Ok(Token::Boolean(true))
    }

    fn consume_false(chars: &mut Peekable<Chars<'_>>) -> Result<Token, LexError> {
        let parsed: Vec<char> = chars.take(5).collect();
        if parsed.len() < 5 {
            return Err(LexError {
                err_type: LexErrorType::UnexpectedEOF,
                row: 0,
                col: 0,
            });
        }
        let compare: Vec<char> = vec!['f', 'a', 'l', 's', 'e'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexError {
                    err_type: LexErrorType::UnexpectedCharacter(parsed[i]),
                    row: 0,
                    col: 0,
                });
            }
        }

        Ok(Token::Boolean(false))
    }

    fn consume_null(chars: &mut Peekable<Chars<'_>>) -> Result<Token, LexError> {
        let parsed: Vec<char> = chars.take(4).collect();
        if parsed.len() < 4 {
            return Err(LexError {
                err_type: LexErrorType::UnexpectedEOF,
                row: 0,
                col: 0,
            });
        }
        let compare: Vec<char> = vec!['n', 'u', 'l', 'l'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexError {
                    err_type: LexErrorType::UnexpectedCharacter(parsed[i]),
                    row: 0,
                    col: 0,
                });
            }
        }

        Ok(Token::Null)
    }
}

#[test]
fn test_lexer_empty_object() {
    let test_string = include_str!("../tests/fixtures/step1/valid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

    assert_eq!(vec![Token::LeftBrace, Token::RightBrace], tokens);
}

#[test]
fn test_lexer_empty_file() {
    let test_string = include_str!("../tests/fixtures/step1/invalid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

    assert_eq!(Vec::<Token>::new(), tokens);
}

#[test]
fn test_lexer_empty_object_with_whitespace() {
    let test_string = "    {   }   ";
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

    assert_eq!(vec![Token::LeftBrace, Token::RightBrace], tokens);
}

#[test]
fn test_lexer_key_value_pair() {
    let test_string = include_str!("../tests/fixtures/step2/valid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

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
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

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
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

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
    let err = Lexer::get_tokens(test_string).err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedCharacter('k'),
            row: 0,
            col: 0,
        },
        err
    );
}

#[test]
fn test_lexer_keywords_and_numbers() {
    let test_string = include_str!("../tests/fixtures/step3/valid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

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
    let err = Lexer::get_tokens(test_string).err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedCharacter('F'),
            row: 0,
            col: 0
        },
        err
    );
}

#[test]
fn test_lexer_with_square_brackets() {
    let test_string = include_str!("../tests/fixtures/step4/valid.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

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
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

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
    let err = Lexer::get_tokens(test_string).err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedCharacter('\''),
            row: 0,
            col: 0
        },
        err
    );
}

#[test]
fn test_lexer_supports_string_with_escape_characters() {
    let test_string = include_str!("../tests/fixtures/step4/valid3.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

    assert_eq!(
        vec![
            Token::LeftBrace,
            Token::StringLiteral(String::from("key")),
            Token::Colon,
            Token::StringLiteral(format!(
                "v\n\u{0008}\u{000C}\r\tal\"ue\"aaaaaa\\/{}{}{}{}{}{}",
                0xaa as char, 0xaa as char, 0xbb as char, 0xbb as char, 0xcc as char, 0xcc as char
            )),
            Token::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_supports_various_number_formats() {
    let test_string = include_str!("../tests/fixtures/step4/valid4.json");
    let tokens: Vec<Token> = Lexer::get_tokens(test_string).unwrap();

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
    let err = Lexer::get_tokens(test_string).err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedEOF,
            row: 0,
            col: 0
        },
        err
    );
}
