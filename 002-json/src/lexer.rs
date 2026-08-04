use std::{iter::Peekable, str::Chars};

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    t_type: TokenType,
    line: usize,
    col: usize,
    index: usize,
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
    index: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LexErrorType {
    UnexpectedToken(char),
    BadNumberFormat(String),
    BadEscapeCharacter(char),
    BadControlCharacterInStringLiteral(char),
    UnexpectedEOF,
    BadHexString(String),
}

pub struct Lexer<'a> {
    input_str: Peekable<Chars<'a>>,
    row: usize,
    col: usize,
    index: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(json_str: &'a str) -> Self {
        Lexer {
            input_str: json_str.chars().peekable(),
            row: 1,
            col: 1,
            index: 0,
        }
    }

    fn peek(&mut self) -> Option<char> {
        let c = self.input_str.peek()?;
        Some(*c)
    }

    fn next(&mut self) -> Option<char> {
        let c = self.input_str.next()?;
        if c == '\n' {
            self.row += 1;
            self.col = 0;
        }

        self.col += 1;
        self.index += 1;

        Some(c)
    }

    pub fn get_tokens(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = vec![];

        while self.peek().is_some() {
            Self::consume_whitespace(self);

            if let Some(c) = self.peek() {
                let line_before = self.row;
                let col_before = self.col;
                let index_before = self.index;

                let token = Self::get_token_or_error(self, c);

                match token {
                    Ok(t) => tokens.push(Token {
                        t_type: t,
                        line: line_before,
                        col: col_before,
                        index: index_before,
                    }),
                    Err(e) => {
                        return Err(LexError {
                            err_type: e,
                            line: self.row,
                            col: self.col,
                            index: self.index,
                        });
                    }
                }
            }
        }

        Ok(tokens)
    }

    fn get_token_or_error(&mut self, c: char) -> Result<TokenType, LexErrorType> {
        let token: Result<TokenType, LexErrorType> = match c {
            '{' => {
                self.next();
                Ok(TokenType::LeftBrace)
            }
            '}' => {
                self.next();
                Ok(TokenType::RightBrace)
            }
            '[' => {
                self.next();
                Ok(TokenType::LeftBracket)
            }
            ']' => {
                self.next();
                Ok(TokenType::RightBracket)
            }
            '"' => Self::consume_string_literal(self),
            ':' => {
                self.next();

                Ok(TokenType::Colon)
            }
            ',' => {
                self.next();
                Ok(TokenType::Comma)
            }
            't' => Self::consume_keyword_true(self),
            'f' => Self::consume_keyword_false(self),
            'n' => Self::consume_keyword_null(self),
            '-' | '1'..='9' => Self::consume_number(self),
            other => Err(LexErrorType::UnexpectedToken(other)),
        };

        token
    }

    fn consume_string_literal(&mut self) -> Result<TokenType, LexErrorType> {
        //Consume first "
        self.next();

        let mut parsed_str = String::new();
        while let Some(c) = self.peek() {
            if c == '"' {
                self.next();
                return Ok(TokenType::StringLiteral(parsed_str));
            } else if c.is_ascii_control() {
                return Err(LexErrorType::BadControlCharacterInStringLiteral(c));
            }
            if c == '\\' {
                self.next();
                if self.peek().is_none() {
                    return Err(LexErrorType::UnexpectedEOF);
                }

                let next_char = self.peek().unwrap();
                match next_char {
                    'n' => {
                        parsed_str.push('\n');
                        self.next();
                    }
                    'b' => {
                        parsed_str.push('\u{0008}');
                        self.next();
                    }
                    'f' => {
                        parsed_str.push('\u{000C}');
                        self.next();
                    }
                    'r' => {
                        parsed_str.push('\r');
                        self.next();
                    }
                    't' => {
                        parsed_str.push('\t');
                        self.next();
                    }
                    'u' => {
                        self.next();
                        let bytes = self.get_hex_string()?;
                        for byte in bytes {
                            parsed_str.push(byte as char);
                        }
                    }
                    '"' | '\\' | '/' => {
                        parsed_str.push(next_char);
                        self.next();
                    }
                    c => {
                        return Err(LexErrorType::BadEscapeCharacter(c));
                    }
                }
            } else {
                parsed_str.push(c);
                self.next();
            }
        }

        Err(LexErrorType::UnexpectedEOF)
    }

    fn get_hex_string(&mut self) -> Result<Vec<u8>, LexErrorType> {
        let hex_chars: Vec<char> = self.input_str.by_ref().take(4).collect();
        if hex_chars.len() < 4 {
            return Err(LexErrorType::UnexpectedEOF);
        }
        let hex_str = format!(
            "{}{}{}{}",
            hex_chars[0], hex_chars[1], hex_chars[2], hex_chars[3]
        );

        match hex::decode(&hex_str) {
            Ok(bytes) => Ok(bytes),
            Err(_) => {
                return Err(LexErrorType::BadHexString(hex_str));
            }
        }
    }

    fn consume_number(&mut self) -> Result<TokenType, LexErrorType> {
        let mut keyword: String = String::new();
        while let Some(c) = self.peek() {
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
            self.next();
        }

        let parse_num = keyword.parse::<f64>();
        parse_num.map_or(Err(LexErrorType::BadNumberFormat(keyword)), |i| {
            Ok(TokenType::Number(i))
        })
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.input_str.peek()
            && c.is_ascii_whitespace()
        {
            self.next();
        }
    }

    fn consume_keyword_true(&mut self) -> Result<TokenType, LexErrorType> {
        let parsed: Vec<char> = self.input_str.by_ref().take(4).collect();
        if parsed.len() < 4 {
            return Err(LexErrorType::UnexpectedEOF);
        }

        let compare: Vec<char> = vec!['t', 'r', 'u', 'e'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexErrorType::UnexpectedToken(parsed[i]));
            }
        }

        Ok(TokenType::Boolean(true))
    }

    fn consume_keyword_false(&mut self) -> Result<TokenType, LexErrorType> {
        let parsed: Vec<char> = self.input_str.by_ref().take(5).collect();
        if parsed.len() < 5 {
            return Err(LexErrorType::UnexpectedEOF);
        }
        let compare: Vec<char> = vec!['f', 'a', 'l', 's', 'e'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexErrorType::UnexpectedToken(parsed[i]));
            }
        }

        Ok(TokenType::Boolean(false))
    }

    fn consume_keyword_null(&mut self) -> Result<TokenType, LexErrorType> {
        let parsed: Vec<char> = self.input_str.by_ref().take(4).collect();
        if parsed.len() < 4 {
            return Err(LexErrorType::UnexpectedEOF);
        }
        let compare: Vec<char> = vec!['n', 'u', 'l', 'l'];
        for i in 0..compare.len() {
            if compare[i] != parsed[i] {
                return Err(LexErrorType::UnexpectedToken(parsed[i]));
            }
        }

        Ok(TokenType::Null)
    }
}

#[test]
fn test_lexer_empty_object() {
    let test_string = include_str!("../tests/fixtures/step1/valid.json");
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(vec![TokenType::LeftBrace, TokenType::RightBrace], tokens);
}

#[test]
fn test_lexer_empty_file() {
    let test_string = include_str!("../tests/fixtures/step1/invalid.json");
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(Vec::<TokenType>::new(), tokens);
}

#[test]
fn test_lexer_empty_object_with_whitespace() {
    let test_string = "    {   }   ";
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
        .unwrap()
        .iter()
        .map(|x| x.t_type.clone())
        .collect();

    assert_eq!(vec![TokenType::LeftBrace, TokenType::RightBrace], tokens);
}

#[test]
fn test_lexer_key_value_pair() {
    let test_string = include_str!("../tests/fixtures/step2/valid.json");
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
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
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
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
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
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
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedToken('k'),
            line: 3,
            col: 3,
            index: 22
        },
        err
    );
}

#[test]
fn test_lexer_keywords_and_numbers() {
    let test_string = include_str!("../tests/fixtures/step3/valid.json");
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
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
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedToken('F'),
            line: 3,
            col: 11,
            index: 24
        },
        err
    );
}

#[test]
fn test_lexer_with_square_brackets() {
    let test_string = include_str!("../tests/fixtures/step4/valid.json");
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
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
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
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
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedToken('\''),
            line: 7,
            col: 13,
            index: 97
        },
        err
    );
}

#[test]
fn test_lexer_supports_string_with_escape_characters() {
    let test_string = include_str!("../tests/fixtures/step4/valid3.json");
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
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
    let mut lexer = Lexer::new(&test_string);
    let tokens: Vec<TokenType> = lexer
        .get_tokens()
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
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedEOF,
            line: 1,
            col: 41,
            index: 40
        },
        err
    );
}

#[test]
fn test_lexer_bad_escape_character() {
    let test_string = "\"bad escape character \\a\"";
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::BadEscapeCharacter('a'),
            line: 1,
            col: 24,
            index: 23
        },
        err
    );
}

#[test]
fn test_lexer_backslash_with_no_character() {
    let test_string = "\"no character escaped \\";
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedEOF,
            line: 1,
            col: 24,
            index: 23
        },
        err
    );
}

#[test]
fn test_lexer_bad_hex_string() {
    let test_string = "\"bad hex string: \\ugggg\"";
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::BadHexString("gggg".to_string()),
            line: 1,
            col: 20,
            index: 19
        },
        err
    );
}

#[test]
fn test_lexer_unexpected_character() {
    let test_string = "{a}";
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::UnexpectedToken('a'),
            line: 1,
            col: 2,
            index: 1
        },
        err
    );
}

#[test]
fn test_lexer_bad_byte_in_string() {
    let test_string = "\"control byte: \nsecond line\"";
    let mut lexer = Lexer::new(&test_string);
    let err = lexer.get_tokens().err().unwrap();

    assert_eq!(
        LexError {
            err_type: LexErrorType::BadControlCharacterInStringLiteral('\n'),
            line: 1,
            col: 16,
            index: 15
        },
        err
    );
}
