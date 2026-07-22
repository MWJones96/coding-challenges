use std::{
    iter::{Enumerate, Peekable},
    str::Chars,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    t_type: TokenType,
    line: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
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
pub struct LexError {
    err_type: LexErrorType,
    line: usize,
    col: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexErrorType {
    UnexpectedCharacter(char),
    BadNumberFormat(String),
    BadEscapeCharacter(char),
    UnexpectedEndOfLine,
    BadHexString(String),
}

pub struct Lexer;
impl Lexer {
    pub fn get_tokens(json_string: &str) -> Result<Vec<Token>, LexError> {
        let mut tokens = vec![];

        for (line_num, line) in json_string.lines().enumerate() {
            let mut chars = line.chars().enumerate().peekable();
            while chars.peek().is_some() {
                Self::consume_whitespace(&mut chars);

                if let Some(&(col_num, c)) = chars.peek() {
                    let token = Self::get_token_or_error(&mut chars, c);

                    match token {
                        Ok(t) => tokens.push(Token {
                            t_type: t,
                            line: line_num + 1,
                            col: col_num + 1,
                        }),
                        Err(e) => {
                            return Err(LexError {
                                err_type: e,
                                line: line_num + 1,
                                col: 0,
                            });
                        }
                    }
                }
            }
        }

        Ok(tokens)
    }

    fn get_token_or_error(
        chars: &mut Peekable<Enumerate<Chars<'_>>>,
        c: char,
    ) -> Result<TokenType, LexErrorType> {
        let token: Result<TokenType, LexErrorType> = match c {
            '{' => {
                chars.next();
                Ok(TokenType::LeftBrace)
            }
            '}' => {
                chars.next();
                Ok(TokenType::RightBrace)
            }
            '[' => {
                chars.next();
                Ok(TokenType::LeftBracket)
            }
            ']' => {
                chars.next();
                Ok(TokenType::RightBracket)
            }
            '"' => Self::consume_string_literal(chars),
            ':' => {
                chars.next();
                Ok(TokenType::Colon)
            }
            ',' => {
                chars.next();
                Ok(TokenType::Comma)
            }
            't' => Self::consume_true(chars),
            'f' => Self::consume_false(chars),
            'n' => Self::consume_null(chars),
            '-' | '1'..='9' => Self::consume_number(chars),
            other => Err(LexErrorType::UnexpectedCharacter(other)),
        };

        token
    }

    fn consume_string_literal(
        chars: &mut Peekable<Enumerate<Chars<'_>>>,
    ) -> Result<TokenType, LexErrorType> {
        //Consume first "
        chars.next();

        let mut parsed_str = String::new();
        while let Some((_, c)) = chars.next() {
            if c == '"' {
                return Ok(TokenType::StringLiteral(parsed_str));
            }
            if c == '\\' {
                if chars.peek().is_none() {
                    return Err(LexErrorType::UnexpectedEndOfLine);
                }

                let (_, next_char) = chars.next().unwrap();
                match next_char {
                    'n' => parsed_str.push('\n'),
                    'b' => parsed_str.push('\u{0008}'),
                    'f' => parsed_str.push('\u{000C}'),
                    'r' => parsed_str.push('\r'),
                    't' => parsed_str.push('\t'),
                    'u' => {
                        let hex_chars: Vec<char> = chars.take(4).map(|(_, c)| c).collect();
                        if hex_chars.len() < 4 {
                            return Err(LexErrorType::UnexpectedEndOfLine);
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
                                return Err(LexErrorType::BadHexString(hex_str));
                            }
                        }
                    }
                    '"' | '\\' | '/' => parsed_str.push(next_char),
                    c => {
                        return Err(LexErrorType::BadEscapeCharacter(c));
                    }
                }
            } else {
                parsed_str.push(c);
            }
        }

        Err(LexErrorType::UnexpectedEndOfLine)
    }

    fn consume_number(
        chars: &mut Peekable<Enumerate<Chars<'_>>>,
    ) -> Result<TokenType, LexErrorType> {
        let mut keyword: String = String::new();
        while let Some(&(_, c)) = chars.peek() {
            if c.is_ascii_whitespace()
                || c == ','
                || c == ':'
                || c == '{'
                || c == '}'
                || c == '['
                || c == ']'
                || c == '"'
            {
                break;
            }
            keyword.push(c);
            chars.next();
        }

        let parse_num = keyword.parse::<f64>();
        parse_num.map_or(Err(LexErrorType::BadNumberFormat(keyword)), |i| {
            Ok(TokenType::Number(i))
        })
    }

    fn consume_whitespace(chars: &mut Peekable<Enumerate<Chars<'_>>>) {
        while let Some(&(_, c)) = chars.peek()
            && c.is_ascii_whitespace()
        {
            chars.next();
        }
    }

    fn consume_true(chars: &mut Peekable<Enumerate<Chars<'_>>>) -> Result<TokenType, LexErrorType> {
        let parsed: Vec<char> = chars.take(4).map(|(_, c)| c).collect();
        if parsed.len() < 4 {
            return Err(LexErrorType::UnexpectedEndOfLine);
        }

        let compare: Vec<char> = vec!['t', 'r', 'u', 'e'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexErrorType::UnexpectedCharacter(parsed[i]));
            }
        }

        Ok(TokenType::Boolean(true))
    }

    fn consume_false(
        chars: &mut Peekable<Enumerate<Chars<'_>>>,
    ) -> Result<TokenType, LexErrorType> {
        let parsed: Vec<char> = chars.take(5).map(|(_, c)| c).collect();
        if parsed.len() < 5 {
            return Err(LexErrorType::UnexpectedEndOfLine);
        }
        let compare: Vec<char> = vec!['f', 'a', 'l', 's', 'e'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexErrorType::UnexpectedCharacter(parsed[i]));
            }
        }

        Ok(TokenType::Boolean(false))
    }

    fn consume_null(chars: &mut Peekable<Enumerate<Chars<'_>>>) -> Result<TokenType, LexErrorType> {
        let parsed: Vec<char> = chars.take(4).map(|(_, c)| c).collect();
        if parsed.len() < 4 {
            return Err(LexErrorType::UnexpectedEndOfLine);
        }
        let compare: Vec<char> = vec!['n', 'u', 'l', 'l'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexErrorType::UnexpectedCharacter(parsed[i]));
            }
        }

        Ok(TokenType::Null)
    }
}

#[test]
fn test_lexer_empty_object() {
    let test_string = include_str!("../tests/fixtures/step1/valid.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(vec![TokenType::LeftBrace, TokenType::RightBrace], tokens);
}

#[test]
fn test_lexer_empty_file() {
    let test_string = include_str!("../tests/fixtures/step1/invalid.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(Vec::<TokenType>::new(), tokens);
}

#[test]
fn test_lexer_empty_object_with_whitespace() {
    let test_string = "    {   }   ";
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(vec![TokenType::LeftBrace, TokenType::RightBrace], tokens);
}

#[test]
fn test_lexer_key_value_pair() {
    let test_string = include_str!("../tests/fixtures/step2/valid.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(
        vec![
            TokenType::LeftBrace,
            TokenType::StringLiteral(String::from("key")),
            TokenType::Colon,
            TokenType::StringLiteral(String::from("value")),
            TokenType::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_multiple_key_value() {
    let test_string = include_str!("../tests/fixtures/step2/valid2.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(
        vec![
            TokenType::LeftBrace,
            TokenType::StringLiteral(String::from("key")),
            TokenType::Colon,
            TokenType::StringLiteral(String::from("value")),
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key2")),
            TokenType::Colon,
            TokenType::StringLiteral(String::from("value")),
            TokenType::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_comma_after_last_key_value() {
    let test_string = include_str!("../tests/fixtures/step2/invalid.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(
        vec![
            TokenType::LeftBrace,
            TokenType::StringLiteral(String::from("key")),
            TokenType::Colon,
            TokenType::StringLiteral(String::from("value")),
            TokenType::Comma,
            TokenType::RightBrace
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
            line: 3,
            col: 0,
        },
        err
    );
}

#[test]
fn test_lexer_keywords_and_numbers() {
    let test_string = include_str!("../tests/fixtures/step3/valid.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(
        vec![
            TokenType::LeftBrace,
            TokenType::StringLiteral(String::from("key1")),
            TokenType::Colon,
            TokenType::Boolean(true),
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key2")),
            TokenType::Colon,
            TokenType::Boolean(false),
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key3")),
            TokenType::Colon,
            TokenType::Null,
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key4")),
            TokenType::Colon,
            TokenType::StringLiteral(String::from("value")),
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key5")),
            TokenType::Colon,
            TokenType::Number(101.0),
            TokenType::RightBrace
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
            line: 3,
            col: 0
        },
        err
    );
}

#[test]
fn test_lexer_with_square_brackets() {
    let test_string = include_str!("../tests/fixtures/step4/valid.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(
        vec![
            TokenType::LeftBrace,
            TokenType::StringLiteral(String::from("key")),
            TokenType::Colon,
            TokenType::StringLiteral(String::from("value")),
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key-n")),
            TokenType::Colon,
            TokenType::Number(101.0),
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key-o")),
            TokenType::Colon,
            TokenType::LeftBrace,
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key-l")),
            TokenType::Colon,
            TokenType::LeftBracket,
            TokenType::RightBracket,
            TokenType::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_with_inner_object_and_array() {
    let test_string = include_str!("../tests/fixtures/step4/valid2.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(
        vec![
            TokenType::LeftBrace,
            TokenType::StringLiteral(String::from("key")),
            TokenType::Colon,
            TokenType::StringLiteral(String::from("value")),
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key-n")),
            TokenType::Colon,
            TokenType::Number(101.0),
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key-o")),
            TokenType::Colon,
            TokenType::LeftBrace,
            TokenType::StringLiteral(String::from("inner key")),
            TokenType::Colon,
            TokenType::StringLiteral(String::from("inner value")),
            TokenType::RightBrace,
            TokenType::Comma,
            TokenType::StringLiteral(String::from("key-l")),
            TokenType::Colon,
            TokenType::LeftBracket,
            TokenType::StringLiteral(String::from("list value")),
            TokenType::RightBracket,
            TokenType::RightBrace
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
            line: 7,
            col: 0
        },
        err
    );
}

#[test]
fn test_lexer_supports_string_with_escape_characters() {
    let test_string = include_str!("../tests/fixtures/step4/valid3.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(
        vec![
            TokenType::LeftBrace,
            TokenType::StringLiteral(String::from("key")),
            TokenType::Colon,
            TokenType::StringLiteral(format!(
                "v\n\u{0008}\u{000C}\r\tal\"ue\"aaaaaa\\/{}{}{}{}{}{}",
                0xaa as char, 0xaa as char, 0xbb as char, 0xbb as char, 0xcc as char, 0xcc as char
            )),
            TokenType::RightBrace
        ],
        tokens
    );
}

#[test]
fn test_lexer_supports_various_number_formats() {
    let test_string = include_str!("../tests/fixtures/step4/valid4.json");
    let tokens: Vec<TokenType> = Lexer::get_tokens(test_string)
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(
        vec![
            TokenType::LeftBracket,
            TokenType::Number(100.0),
            TokenType::Comma,
            TokenType::Number(100.5),
            TokenType::Comma,
            TokenType::Number(100.0),
            TokenType::Comma,
            TokenType::Number(100.0),
            TokenType::Comma,
            TokenType::Number(-1.5),
            TokenType::Comma,
            TokenType::Number(-1.0),
            TokenType::RightBracket,
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
            err_type: LexErrorType::UnexpectedEndOfLine,
            line: 1,
            col: 0
        },
        err
    );
}

#[test]
fn test_lexer_bad_escape_character() {
    let test_string = "\"bad escape character \\a\"";
    let err = Lexer::get_tokens(test_string).err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::BadEscapeCharacter('a'),
            line: 1,
            col: 0
        },
        err
    );
}

#[test]
fn test_lexer_backslash_with_no_character() {
    let test_string = "\"no character escaped \\";
    let err = Lexer::get_tokens(test_string).err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedEndOfLine,
            line: 1,
            col: 0
        },
        err
    );
}

#[test]
fn test_lexer_bad_hex_string() {
    let test_string = "\"bad hex string: \\ugggg\"";
    let err = Lexer::get_tokens(test_string).err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::BadHexString("gggg".to_string()),
            line: 1,
            col: 0
        },
        err
    );
}

#[test]
fn test_lexer_unexpected_character() {
    let test_string = "{a}";
    let err = Lexer::get_tokens(test_string).err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedCharacter('a'),
            line: 1,
            col: 0
        },
        err
    );
}
