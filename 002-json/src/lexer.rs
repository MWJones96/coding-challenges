use std::{collections::HashSet, iter::Peekable, str::Chars};

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
    UnexpectedToken(String),
    BadNumberFormat(String),
    BadEscapeCharacter(char),
    MissingEscapeCharacter(String),
    BadControlCharacterInStringLiteral(char),
    UnterminatedString(String),
    BadEscapedHexString(String),
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
                            line: line_before,
                            col: col_before,
                            index: index_before,
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
            '-' | '0'..='9' => Self::consume_number(self),
            _kwd => {
                let kwd = Self::consume_keyword(self);
                match kwd.as_str() {
                    "true" => Ok(TokenType::Boolean(true)),
                    "false" => Ok(TokenType::Boolean(false)),
                    "null" => Ok(TokenType::Null),
                    _ => Err(LexErrorType::UnexpectedToken(kwd)),
                }
            }
        };

        token
    }

    fn consume_string_literal(&mut self) -> Result<TokenType, LexErrorType> {
        self.next();

        let mut parsed_str = String::new();
        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.next();
                    return Ok(TokenType::StringLiteral(parsed_str));
                }
                '\x00'..='\x1F' | '\x7F' => {
                    return Err(LexErrorType::BadControlCharacterInStringLiteral(c));
                }
                '\\' => {
                    self.next();
                    let next_char = self
                        .peek()
                        .ok_or_else(|| LexErrorType::MissingEscapeCharacter(parsed_str.clone()))?;
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
                }
                _ => {
                    parsed_str.push(c);
                    self.next();
                }
            }
        }

        Err(LexErrorType::UnterminatedString(parsed_str))
    }

    fn get_hex_string(&mut self) -> Result<Vec<u8>, LexErrorType> {
        let mut hex_str: String = String::new();

        for _ in 0..4 {
            if let Some(c) = self.peek() {
                hex_str.push(c);
                self.next();
            } else {
                return Err(LexErrorType::BadEscapedHexString(hex_str));
            }
        }

        match hex::decode(&hex_str) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(LexErrorType::BadEscapedHexString(hex_str)),
        }
    }

    fn consume_number(&mut self) -> Result<TokenType, LexErrorType> {
        let delimeters = HashSet::from([',', ':', '{', '}', '[', ']', '"']);

        let mut num_string: String = String::new();
        while let Some(c) = self.peek() {
            if delimeters.contains(&c) || c.is_ascii_whitespace() {
                break;
            }

            num_string.push(c);
            self.next();
        }

        let parse_num = num_string.parse::<f64>();
        parse_num.map_or(Err(LexErrorType::BadNumberFormat(num_string)), |i| {
            Ok(TokenType::Number(i))
        })
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_ascii_whitespace() {
                break;
            }

            self.next();
        }
    }

    fn consume_keyword(&mut self) -> String {
        let delimeters = HashSet::from([',', ':', '{', '}', '[', ']', '"']);

        let mut parsed = String::new();
        while let Some(c) = self.peek() {
            if delimeters.contains(&c) || c.is_ascii_whitespace() {
                break;
            }

            parsed.push(c);
            self.next();
        }

        parsed
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lexer_empty_file() {
        let test_string = "";
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
    fn test_lexer_with_square_brackets() {
        let test_string = include_str!("../tests/fixtures/valid.json");
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
                TokenType::StringLiteral("key".to_string()),
                TokenType::Colon,
                TokenType::StringLiteral("value".to_string()),
                TokenType::Comma,
                TokenType::StringLiteral("key-n".to_string()),
                TokenType::Colon,
                TokenType::Number(101.0),
                TokenType::Comma,
                TokenType::StringLiteral("key-o".to_string()),
                TokenType::Colon,
                TokenType::LeftBrace,
                TokenType::StringLiteral("inner key".to_string()),
                TokenType::Colon,
                TokenType::StringLiteral("inner value".to_string()),
                TokenType::RightBrace,
                TokenType::Comma,
                TokenType::StringLiteral("key-l".to_string()),
                TokenType::Colon,
                TokenType::LeftBracket,
                TokenType::StringLiteral("list value".to_string()),
                TokenType::Comma,
                TokenType::Boolean(true),
                TokenType::Comma,
                TokenType::Boolean(false),
                TokenType::Comma,
                TokenType::Null,
                TokenType::RightBracket,
                TokenType::RightBrace
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
                err_type: LexErrorType::UnterminatedString(
                    "this string is not terminated correctly".to_string()
                ),
                line: 1,
                col: 1,
                index: 0
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
                col: 1,
                index: 0
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
                err_type: LexErrorType::MissingEscapeCharacter("no character escaped ".to_string()),
                line: 1,
                col: 1,
                index: 0
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
                err_type: LexErrorType::BadEscapedHexString("gggg".to_string()),
                line: 1,
                col: 1,
                index: 0
            },
            err
        );
    }

    #[test]
    fn test_lexer_unexpected_character() {
        let test_string = "{abcdef}";
        let mut lexer = Lexer::new(&test_string);
        let err = lexer.get_tokens().err().unwrap();

        assert_eq!(
            LexError {
                err_type: LexErrorType::UnexpectedToken("abcdef".to_string()),
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
                col: 1,
                index: 0
            },
            err
        );
    }
}
