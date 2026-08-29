use std::{iter::Peekable, str::Chars, sync::LazyLock};

use regex::Regex;

static NUMBER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?$").unwrap());

fn is_delimiter(c: char) -> bool {
    matches!(c, ',' | ':' | '{' | '}' | '[' | ']' | '"')
}

/// JSON's insignificant whitespace (RFC 8259): space, tab, LF, CR. Notably
/// narrower than `char::is_ascii_whitespace`, which also accepts form feed.
fn is_json_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

/// The `TokenType` for a punctuation character that needs no further lexing.
fn single_char_token(c: char) -> Option<TokenType> {
    match c {
        '{' => Some(TokenType::LeftBrace),
        '}' => Some(TokenType::RightBrace),
        '[' => Some(TokenType::LeftBracket),
        ']' => Some(TokenType::RightBracket),
        ':' => Some(TokenType::Colon),
        ',' => Some(TokenType::Comma),
        _ => None,
    }
}

/// The character a single-letter `\` escape (`\n`, `\"`, etc.) stands for.
/// Doesn't handle `\u`, which needs more than one character to resolve.
fn simple_escape(c: char) -> Option<char> {
    match c {
        'n' => Some('\n'),
        'b' => Some('\u{0008}'),
        'f' => Some('\u{000C}'),
        'r' => Some('\r'),
        't' => Some('\t'),
        '"' | '\\' | '/' => Some(c),
        _ => None,
    }
}

/// One lexical unit of a JSON document, tagged with where it starts.
#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    t_type: TokenType,
    line: usize,
    col: usize,
    index: usize,
}

impl Token {
    pub fn t_type(&self) -> &TokenType {
        &self.t_type
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

/// The kind of a `Token` and any value it carries.
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

/// A lexing failure, tagged with where in the input it started.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct LexError {
    err_type: LexErrorType,
    line: usize,
    col: usize,
    index: usize,
}

impl LexError {
    pub fn err_type(&self) -> &LexErrorType {
        &self.err_type
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

/// Why lexing failed, independent of where.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LexErrorType {
    UnexpectedToken(String),
    BadNumber(String),
    BadEscapeCharacter(char),
    ControlCharacterInStringLiteral(char),
    UnterminatedString(String),
    BadHexString(String),
}

/// A snapshot of where the lexer was at some earlier point, so an error
/// or token can be tagged with the position it actually started at instead
/// of wherever the lexer happens to be by the time the mistake is noticed.
#[derive(Clone, Copy)]
struct Position {
    line: usize,
    col: usize,
    index: usize,
}

/// Turns a JSON document's source text into a stream of `Token`s.
pub struct Lexer<'a> {
    input_str: Peekable<Chars<'a>>,
    line: usize,
    col: usize,
    index: usize,
}

impl<'a> Lexer<'a> {
    /// Creates a lexer positioned at the start of `json_str`.
    pub fn new(json_str: &'a str) -> Self {
        Lexer {
            input_str: json_str.chars().peekable(),
            line: 1,
            col: 1,
            index: 0,
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.input_str.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.input_str.next()?;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.index += 1;

        Some(c)
    }

    fn position(&self) -> Position {
        Position {
            line: self.line,
            col: self.col,
            index: self.index,
        }
    }

    /// Builds an error anchored to the lexer's *current* position - for
    /// mistakes that are noticed before the offending character is consumed.
    fn error(&self, err_type: LexErrorType) -> LexError {
        self.error_at(self.position(), err_type)
    }

    /// Builds an error anchored to a position captured earlier - for mistakes
    /// that are only noticed after consuming past where they actually started.
    fn error_at(&self, pos: Position, err_type: LexErrorType) -> LexError {
        LexError {
            err_type,
            line: pos.line,
            col: pos.col,
            index: pos.index,
        }
    }

    /// Lexes the whole input, stopping at the first error.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = vec![];

        loop {
            self.consume_whitespace();

            let Some(c) = self.peek() else { break };
            let start = self.position();

            let t_type = self.next_token(c)?;
            tokens.push(Token {
                t_type,
                line: start.line,
                col: start.col,
                index: start.index,
            });
        }

        Ok(tokens)
    }

    fn next_token(&mut self, c: char) -> Result<TokenType, LexError> {
        let start = self.position();

        if let Some(t_type) = single_char_token(c) {
            self.advance();
            return Ok(t_type);
        }

        match c {
            '"' => self.consume_string_literal(start),
            '-' | '+' | '.' | 'e' | 'E' | '0'..='9' => {
                let num = self.consume_until_delimiter();

                if !NUMBER_RE.is_match(&num) {
                    return Err(self.error_at(start, LexErrorType::BadNumber(num)));
                }

                num.parse::<f64>()
                    .map(TokenType::Number)
                    .map_err(|_| self.error_at(start, LexErrorType::BadNumber(num)))
            }
            _ => {
                let kwd = self.consume_until_delimiter();
                match kwd.as_str() {
                    "true" => Ok(TokenType::Boolean(true)),
                    "false" => Ok(TokenType::Boolean(false)),
                    "null" => Ok(TokenType::Null),
                    _ => Err(self.error_at(start, LexErrorType::UnexpectedToken(kwd))),
                }
            }
        }
    }

    fn consume_string_literal(&mut self, start: Position) -> Result<TokenType, LexError> {
        self.advance();

        let mut parsed_str = String::new();
        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.advance();
                    return Ok(TokenType::StringLiteral(parsed_str));
                }
                '\x00'..='\x1F' => {
                    return Err(self.error(LexErrorType::ControlCharacterInStringLiteral(c)));
                }
                '\\' => {
                    self.advance();
                    let next_char = self.peek().ok_or_else(|| {
                        self.error_at(start, LexErrorType::UnterminatedString(parsed_str.clone()))
                    })?;

                    if let Some(escaped) = simple_escape(next_char) {
                        parsed_str.push(escaped);
                        self.advance();
                    } else if next_char == 'u' {
                        self.advance();
                        let ch = self.parse_unicode_escape()?;
                        parsed_str.push(ch);
                    } else {
                        return Err(self.error(LexErrorType::BadEscapeCharacter(next_char)));
                    }
                }
                _ => {
                    parsed_str.push(c);
                    self.advance();
                }
            }
        }

        Err(self.error_at(start, LexErrorType::UnterminatedString(parsed_str)))
    }

    /// Parses a `\uXXXX` escape into a single `char`, transparently combining
    /// a UTF-16 surrogate pair (`\uD800`-`\uDBFF` followed by `\uDC00`-`\uDFFF`)
    /// into one scalar value when the first code unit demands it.
    fn parse_unicode_escape(&mut self) -> Result<char, LexError> {
        let start = self.position();
        let high = self.parse_hex_code_unit()?;

        if (0xD800..=0xDBFF).contains(&high) {
            if self.peek() != Some('\\') {
                return Err(self.error_at(start, LexErrorType::BadHexString(format!("{high:04x}"))));
            }
            self.advance();

            if self.peek() != Some('u') {
                return Err(self.error_at(start, LexErrorType::BadHexString(format!("{high:04x}"))));
            }
            self.advance();

            let low_start = self.position();
            let low = self.parse_hex_code_unit()?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return Err(
                    self.error_at(low_start, LexErrorType::BadHexString(format!("{low:04x}")))
                );
            }

            let combined = 0x10000 + ((high as u32 - 0xD800) << 10) + (low as u32 - 0xDC00);
            return char::from_u32(combined).ok_or_else(|| {
                self.error_at(start, LexErrorType::BadHexString(format!("{combined:x}")))
            });
        }

        char::from_u32(high as u32)
            .ok_or_else(|| self.error_at(start, LexErrorType::BadHexString(format!("{high:04x}"))))
    }

    fn parse_hex_code_unit(&mut self) -> Result<u16, LexError> {
        let start = self.position();
        let hex_str = self.consume_hex_string();
        if hex_str.len() < 4 {
            return Err(self.error_at(start, LexErrorType::BadHexString(hex_str)));
        }

        u16::from_str_radix(&hex_str, 16)
            .map_err(|_| self.error_at(start, LexErrorType::BadHexString(hex_str)))
    }

    fn consume_hex_string(&mut self) -> String {
        let mut hex_str: String = String::new();

        for _ in 0..4 {
            if let Some(c) = self.peek()
                && !is_delimiter(c)
                && !is_json_whitespace(c)
            {
                hex_str.push(c);
                self.advance();
            } else {
                break;
            }
        }

        hex_str
    }

    /// Consumes characters up to the next structural delimiter or whitespace -
    /// shared by number and keyword scanning, which only differ in what they
    /// do with the resulting slice.
    fn consume_until_delimiter(&mut self) -> String {
        let mut consumed = String::new();
        while let Some(c) = self.peek() {
            if is_delimiter(c) || is_json_whitespace(c) {
                break;
            }

            consumed.push(c);
            self.advance();
        }

        consumed
    }

    fn consume_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !is_json_whitespace(c) {
                break;
            }

            self.advance();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Lexes `input` and returns just the token kinds, for tests that don't
    /// care about position info.
    fn token_types(input: &str) -> Vec<TokenType> {
        Lexer::new(input)
            .tokenize()
            .unwrap()
            .into_iter()
            .map(|t| t.t_type)
            .collect()
    }

    #[test]
    fn test_lexer_empty_file() {
        assert_eq!(Vec::<TokenType>::new(), token_types(""));
    }

    #[test]
    fn test_lexer_full_document() {
        let tokens = token_types(include_str!("../tests/fixtures/valid.json"));

        assert_eq!(
            vec![
                TokenType::LeftBrace,
                TokenType::StringLiteral("key".to_string()),
                TokenType::Colon,
                TokenType::StringLiteral("value \n\x08\x0C\r\t\"\\/".to_string()),
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
                TokenType::StringLiteral("inner value \u{1234} \u{aaaa} \u{aaaa}".to_string()),
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
    fn test_lexer_get_tokens_with_index_info() {
        let mut lexer = Lexer::new(" [\n  ]");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            vec![
                Token {
                    t_type: TokenType::LeftBracket,
                    line: 1,
                    col: 2,
                    index: 1
                },
                Token {
                    t_type: TokenType::RightBracket,
                    line: 2,
                    col: 3,
                    index: 5
                },
            ],
            tokens
        );
    }

    #[test]
    fn test_lexer_supports_unterminated_string() {
        let mut lexer = Lexer::new("\"this string is not terminated correctly");
        let err = lexer.tokenize().err().unwrap();

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
        let mut lexer = Lexer::new("\"bad escape character \\a\"");
        let err = lexer.tokenize().err().unwrap();

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
        let mut lexer = Lexer::new("\"no character escaped \\");
        let err = lexer.tokenize().err().unwrap();

        assert_eq!(
            LexError {
                err_type: LexErrorType::UnterminatedString("no character escaped ".to_string()),
                line: 1,
                col: 1,
                index: 0
            },
            err
        );
    }

    #[test]
    fn test_lexer_bad_hex_string() {
        let mut lexer = Lexer::new("\"bad hex string: \\ugggg\"");
        let err = lexer.tokenize().err().unwrap();

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
        let mut lexer = Lexer::new("{abcdef}");
        let err = lexer.tokenize().err().unwrap();

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
        let mut lexer = Lexer::new("\"control byte: \nsecond line\"");
        let err = lexer.tokenize().err().unwrap();

        assert_eq!(
            LexError {
                err_type: LexErrorType::ControlCharacterInStringLiteral('\n'),
                line: 1,
                col: 16,
                index: 15
            },
            err
        );
    }

    #[test]
    fn test_lexer_short_hex_string() {
        let mut lexer = Lexer::new("[\"\\ufff \"]");
        let err = lexer.tokenize().err().unwrap();
        assert_eq!(
            LexError {
                err_type: LexErrorType::BadHexString("fff".to_string()),
                line: 1,
                col: 5,
                index: 4
            },
            err
        );
    }

    #[test]
    fn test_lexer_valid_numbers() {
        let tokens = token_types("42 -17 0 -0 0.1 1.23456 6.02e+23 1e10 -1.5E-4");

        assert_eq!(
            vec![
                TokenType::Number(42.0),
                TokenType::Number(-17.0),
                TokenType::Number(0.0),
                TokenType::Number(0.0),
                TokenType::Number(0.1),
                TokenType::Number(1.23456),
                TokenType::Number(6.02e23),
                TokenType::Number(1e10),
                TokenType::Number(-1.5e-4),
            ],
            tokens
        );
    }

    #[test]
    fn test_lexer_invalid_number_formats() {
        let cases = ["+0", "0123", "12.", ".5", "1e"];

        for input in cases {
            let err = Lexer::new(input).tokenize().err().unwrap();
            assert_eq!(LexErrorType::BadNumber(input.to_string()), err.err_type);
        }
    }

    #[test]
    fn test_lexer_unicode_escape_bmp_character() {
        let mut lexer = Lexer::new("\"\\u1234\"");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            vec![Token {
                t_type: TokenType::StringLiteral("\u{1234}".to_string()),
                line: 1,
                col: 1,
                index: 0
            }],
            tokens
        );
    }

    #[test]
    fn test_lexer_unicode_escape_surrogate_pair() {
        let mut lexer = Lexer::new("\"\\uD83D\\uDE00\"");
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            vec![Token {
                t_type: TokenType::StringLiteral("\u{1F600}".to_string()),
                line: 1,
                col: 1,
                index: 0
            }],
            tokens
        );
    }

    #[test]
    fn test_lexer_unicode_escape_lone_high_surrogate() {
        let mut lexer = Lexer::new("\"\\uD800\"");
        let err = lexer.tokenize().err().unwrap();

        assert_eq!(LexErrorType::BadHexString("d800".to_string()), err.err_type);
    }

    #[test]
    fn test_lexer_hex_escape_stops_at_delimiter() {
        let mut lexer = Lexer::new("\"\\u12\"");
        let err = lexer.tokenize().err().unwrap();

        assert_eq!(
            LexError {
                err_type: LexErrorType::BadHexString("12".to_string()),
                line: 1,
                col: 4,
                index: 3
            },
            err
        );
    }

    #[test]
    fn test_lexer_form_feed_is_not_whitespace() {
        let err = Lexer::new("{\u{0C}}").tokenize().err().unwrap();

        assert_eq!(
            LexErrorType::UnexpectedToken("\u{0C}".to_string()),
            err.err_type
        );
    }

    #[test]
    fn test_lexer_allows_del_byte_in_string() {
        let tokens = token_types("\"a\u{7F}b\"");

        assert_eq!(
            vec![TokenType::StringLiteral("a\u{7F}b".to_string())],
            tokens
        );
    }
}
