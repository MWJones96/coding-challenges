#[derive(Debug, PartialEq)]
enum Token {
    LeftBrace,
    RightBrace,
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
                    chars.next();
                }
                '}' => {
                    tokens.push(Token::RightBrace);
                    chars.next();
                }
                _ => todo!(),
            }
        }

        tokens
    }
}

#[test]
fn test_lexer_empty_object() {
    let test_string = "{}";
    let tokens: Vec<Token> = Lexer::get_tokens(test_string);

    assert_eq!(vec![Token::LeftBrace, Token::RightBrace], tokens);
}
