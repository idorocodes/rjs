#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Identifier(String),

    // Keywords
    Let,
    Const,
    If,
    Else,
    While,
    For,
    Function,
    
    Return,
    Break,
    Continue,
    True,
    False,
    Null,
    Undefined,

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Equals,        // =
    EqualsEquals,  // ==
    StrictEquals,  // ===
    NotEquals,     // !=
    Less,          //
    Greater,       // >
    LessEquals,    // <=
    GreaterEquals, // >=
    AndAnd,        // &&
    OrOr,          // ||
    Bang,          // !

    // Punctuation
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Comma,
    Dot,

    // End of input
    Eof,
}


pub fn tokenizer(input: String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(ch) = iter.next() {
        match ch {
            ch if ch.is_whitespace() => continue,
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            '+' => tokens.push(Token::Plus),
            '-' => tokens.push(Token::Minus),
            '*' => tokens.push(Token::Star),
            '/' => tokens.push(Token::Slash),
            ';' => tokens.push(Token::Semicolon),
            ',' => tokens.push(Token::Comma),
            '.' => tokens.push(Token::Dot),
            '{' => tokens.push(Token::LeftBrace),
            '}' => tokens.push(Token::RightBrace),
            '[' => tokens.push(Token::LeftBracket),
            ']' => tokens.push(Token::RightBracket),

            '=' => {
                if iter.peek() == Some(&'=') {
                    iter.next();
                    if iter.peek() == Some(&'=') {
                        iter.next();
                        tokens.push(Token::StrictEquals);
                    } else {
                        tokens.push(Token::EqualsEquals);
                    }
                } else {
                    tokens.push(Token::Equals);
                }
            }

            '<' => {
                if iter.peek() == Some(&'=') {
                    iter.next();
                    tokens.push(Token::LessEquals);
                } else {
                    tokens.push(Token::Less);
                }
            }

            '>' => {
                if iter.peek() == Some(&'=') {
                    iter.next();
                    tokens.push(Token::GreaterEquals);
                } else {
                    tokens.push(Token::Greater);
                }
            }

            '&' => {
                if iter.peek() == Some(&'&') {
                    iter.next();
                    tokens.push(Token::AndAnd);
                } else {
                    panic!("Unexpected solo '&' operator");
                }
            }
            '!' => {
                if iter.peek() == Some(&'=') {
                    iter.next(); // Consume '='
                    tokens.push(Token::NotEquals);
                } else {
                    tokens.push(Token::Bang);
                }
            }
            '|' => {
                if iter.peek() == Some(&'|') {
                    iter.next();
                    tokens.push(Token::OrOr);
                } else {
                    panic!("Unexpected solo '|' operator");
                }
            }

            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                ident.push(ch);

                while let Some(&next_ch) = iter.peek() {
                    if next_ch.is_alphanumeric() || next_ch == '_' {
                        ident.push(iter.next().unwrap());
                    } else {
                        break;
                    }
                }

                match ident.as_str() {
                    "let" => tokens.push(Token::Let),
                    "const" => tokens.push(Token::Const),
                    "if" => tokens.push(Token::If),
                    "else" => tokens.push(Token::Else),
                    "function" => tokens.push(Token::Function),
                    "for" => tokens.push(Token::For),
                    "return" => tokens.push(Token::Return),
                    "break" => tokens.push(Token::Break),
                    "continue" => tokens.push(Token::Continue),
                    "true" => tokens.push(Token::True),
                    "false" => tokens.push(Token::False),
                    "while" => tokens.push(Token::While),
                    "null" => tokens.push(Token::Null),
                    "undefined" => tokens.push(Token::Undefined),
                    _ => tokens.push(Token::Identifier(ident)),
                }
            }

            '"' => {
                let mut string_content = String::new();
                while let Some(&next_ch) = iter.peek() {
                    if next_ch == '"' {
                        iter.next();
                        break;
                    } else {
                        string_content.push(iter.next().unwrap());
                    }
                }
                tokens.push(Token::String(string_content));
            }

            '0'..='9' => {
                let mut num_str = String::new();
                num_str.push(ch);

                while let Some(&next_ch) = iter.peek() {
                    if next_ch.is_ascii_digit() || next_ch == '.' {
                        num_str.push(iter.next().unwrap());
                    } else {
                        break;
                    }
                }

                let num: f64 = num_str.parse().expect("Failed to parse number");
                tokens.push(Token::Number(num));
            }
            _ => {
                panic!("unrecognized char: {}", ch);
            }
        }
    }

    tokens.push(Token::Eof);
    tokens
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords_and_identifiers() {
        let input = "let x = 42; const name = \"js\"; if (true) { return null; }".to_string();
        let tokens = tokenizer(input);

        
        let expected = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Equals,
            Token::Number(42.0),
            Token::Semicolon,
            Token::Const,
            Token::Identifier("name".to_string()),
            Token::Equals,
            Token::String("js".to_string()),
            Token::Semicolon,
            Token::If,
            Token::LeftParen,
            Token::True,
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::Null,
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn test_operators() {
        let input = "1 === 1 && 2 >= 1 || 3 != 4".to_string();
        let tokens = tokenizer(input);

        let expected = vec![
            Token::Number(1.0),
            Token::StrictEquals,
            Token::Number(1.0),
            Token::AndAnd,
            Token::Number(2.0),
            Token::GreaterEquals,
            Token::Number(1.0),
            Token::OrOr,
            Token::Number(3.0),
            Token::Identifier("!".to_string()), 
            Token::Equals,
            Token::Number(4.0),
            Token::Eof,
        ];
        
        assert_eq!(tokens[0..9], expected[0..9]);
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenizer("   \n\t  ".to_string());
        assert_eq!(tokens, vec![Token::Eof]);
    }

    #[test]
    #[should_panic(expected = "Unexpected solo '&' operator")]
    fn test_invalid_operator() {
        tokenizer("&".to_string());
    }
}
