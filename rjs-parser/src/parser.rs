use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}
#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Self { tokens, pos: 0 }
    }

    pub fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub fn advance(&mut self) -> Token {
        if self.tokens[self.pos] == Token::Eof {
            return Token::Eof;
        }

        let current_token = self.tokens[self.pos].clone();
        self.pos += 1;
        current_token
    }
    pub fn check(&self, token: Token) -> bool {
        *self.peek() == token
    }

    pub fn expect(&mut self, token: Token) -> Result<(), ParseError> {
        if self.check(token.clone()) {
            self.advance();
            Ok(())
        } else {
            return Err(ParseError {
                message: format!("Unable to parse token: {:?}", token),
            });
        }
    }

    pub fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.peek().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Number(n))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expr::String(s))
            }
            Token::Identifier(id) => {
                self.advance();
                Ok(Expr::Identifier(id))
            }
            Token::True => {
                self.advance();
                Ok(Expr::Boolean(true))
            }
            Token::False => {
                self.advance();
                Ok(Expr::Boolean(false))
            }
            Token::Null => {
                self.advance();
                Ok(Expr::Null)
            }
            Token::Undefined => {
                self.advance();
                Ok(Expr::Undefined)
            }
            _ => Err(ParseError {
                message: format!("Cannot parse unexpected token: {:?}", self.peek()),
            }),
        }
    }

    pub fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.check(Token::Let) {
            self.advance();

            let name = match self.advance() {
                Token::Identifier(id) => id,
                other => {
                    return Err(ParseError {
                        message: format!("Expected variable name after 'let', found {:?}", other),
                    });
                }
            };

            self.expect(Token::Equals)?; // consume '='
            let value = self.expression()?;
            self.expect(Token::Semicolon)?; // consume ';'

            Ok(Stmt::Let { name, value })
        } else {
            let expr = self.expression()?;
            self.expect(Token::Semicolon)?;
            Ok(Stmt::ExprStmt(expr))
        }
    }

    pub fn multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.check(Token::Star) || self.check(Token::Slash) {
            let operator_token = self.advance();
            let op = match operator_token {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub fn logical_and(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;

        while self.check(Token::AndAnd) {
            let token = self.advance();
            let right = self.comparison()?;

            let op = match token {
                Token::AndAnd => BinOp::And,
                _ => unreachable!(),
            };
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            }
        }

        Ok(expr)
    }

    pub fn logical_or(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.logical_and()?;

        while self.check(Token::OrOr) {
            self.advance();
            let right = self.logical_and()?;
            expr = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.additive()?;

        while self.check(Token::EqualsEquals)
            || self.check(Token::StrictEquals)
            || self.check(Token::NotEquals)
            || self.check(Token::Less)
            || self.check(Token::Greater)
            || self.check(Token::LessEquals)
            || self.check(Token::GreaterEquals)
        {
            let token = self.advance();
            let op = match token {
                Token::EqualsEquals => BinOp::Eq,
                Token::StrictEquals => BinOp::StrictEq,
                Token::NotEquals => BinOp::NotEq,
                Token::Less => BinOp::Lt,
                Token::Greater => BinOp::Gt,
                Token::LessEquals => BinOp::LtEq,
                Token::GreaterEquals => BinOp::GtEq,
                _ => unreachable!(),
            };

            let right = self.additive()?;

            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    pub fn additive(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.multiplicative()?;

        while self.check(Token::Plus) || self.check(Token::Minus) {
            let operator_token = self.advance();
            let op = match operator_token {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            let right = self.multiplicative()?;
            expr = Expr::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
            };
        }

        Ok(expr)
    }
    pub fn postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;

        while self.check(Token::LeftParen)
            || self.check(Token::LeftBracket)
            || self.check(Token::Dot)
        {
            let token = self.advance();

            expr = match token {
                Token::Dot => {
                    let property = match self.advance() {
                        Token::Identifier(name) => name,
                        other => {
                            return Err(ParseError {
                                message: format!(
                                    "Expected property name after '.', found {:?}",
                                    other
                                ),
                            });
                        }
                    };

                    Expr::Member {
                        object: Box::new(expr),
                        property,
                    }
                }

                Token::LeftBracket => {
                    let index = self.expression()?;
                    self.expect(Token::RightBracket)?;

                    Expr::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    }
                }

                Token::LeftParen => {
                    let mut args = Vec::new();

                    if !self.check(Token::RightParen) {
                        args.push(self.expression()?);

                        while self.check(Token::Comma) {
                            self.advance(); // consume the comma
                            args.push(self.expression()?);
                        }
                    }

                    self.expect(Token::RightParen)?;

                    Expr::Call {
                        callee: Box::new(expr),
                        args,
                    }
                }

                _ => unreachable!(), // the while condition already guarantees one of the three above
            };
        }

        Ok(expr)
    }

    pub fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.check(Token::Minus) || self.check(Token::Bang) {
            let current = self.advance();
            let operand = self.unary()?;

            let op = match current {
                Token::Minus => UnaryOp::Neg,
                Token::Bang => UnaryOp::Not,
                _ => unreachable!(),
            };

            Ok(Expr::Unary {
                op,
                operand: Box::new(operand),
            })
        } else {
            self.postfix()
        }
    }
    pub fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.logical_or()?;

        if self.check(Token::Equals) {
            let name = match expr {
                Expr::Identifier(name) => name,
                _ => {
                    return Err(ParseError {
                        message: "Invalid assignment target".to_string(),
                    });
                }
            };

            self.advance(); // consume '='
            let value = self.assignment()?; // right-associative

            Ok(Expr::Assign {
                name,
                value: Box::new(value),
            })
        } else {
            Ok(expr)
        }
    }
    pub fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    pub fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        self.advance();
        let mut stmt: Vec<Stmt> = Vec::new();

        while !self.check(Token::RightBrace) && !self.check(Token::Eof) {
            stmt.push(self.statement()?);
        }
        self.expect(Token::RightBrace)?;

        Ok(stmt)
    }

    pub fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        self.expect(Token::LeftParen)?;
        let cond = self.expression()?;
        self.expect(Token::RightParen)?;

        let then_branch = self.statement()?;

        let else_branch = if self.check(Token::Else) {
            self.advance();
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            cond,
            then_branch: Box::new(then_branch),
            else_branch,
        })
    }

    pub fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        self.expect(Token::LeftParen)?;
        let cond = self.expression()?;
        self.expect(Token::RightParen)?;

        let body = self.statement()?;
        Ok(Stmt::While {
            cond,
            body: Box::new(body),
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Identifier(String),

    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },

    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Member {
        object: Box<Expr>,
        property: String,
    }, // user.name
    Index {
        object: Box<Expr>,
        index: Box<Expr>,
    }, // nums[i]

    ObjectLiteral(Vec<(String, Expr)>),
    ArrayLiteral(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Expr,
    },
    Const {
        name: String,
        value: Expr,
    },
    ExprStmt(Expr),
    Block(Vec<Stmt>),

    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        cond: Expr,
        body: Box<Stmt>,
    },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
    },

    Return(Option<Expr>),
    Break,
    Continue,
    FunctionDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, // -x
    Not, // !x
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    StrictEq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_primary_literals() {
        let tokens = vec![Token::String("hello".to_string()), Token::Eof];
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.primary().unwrap(), Expr::String("hello".to_string()));

        let tokens = vec![Token::Identifier("myVar".to_string()), Token::Eof];
        let mut parser = Parser::new(tokens);
        assert_eq!(
            parser.primary().unwrap(),
            Expr::Identifier("myVar".to_string())
        );

        let tokens = vec![Token::True, Token::Eof];
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.primary().unwrap(), Expr::Boolean(true));

        let tokens = vec![Token::Null, Token::Eof];
        let mut parser = Parser::new(tokens);
        assert_eq!(parser.primary().unwrap(), Expr::Null);
    }

    #[test]
    fn test_binary_precedence() {
        let tokens = vec![
            Token::Number(5.0),
            Token::Plus,
            Token::Number(10.0),
            Token::Star,
            Token::Number(2.0),
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);
        let expected = Expr::Binary {
            op: BinOp::Add,
            left: Box::new(Expr::Number(5.0)),
            right: Box::new(Expr::Binary {
                op: BinOp::Mul,
                left: Box::new(Expr::Number(10.0)),
                right: Box::new(Expr::Number(2.0)),
            }),
        };
        assert_eq!(parser.expression().unwrap(), expected);
    }

    #[test]
    fn test_let_statement() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("score".to_string()),
            Token::Equals,
            Token::Number(100.0),
            Token::Semicolon,
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.statement().unwrap();

        match result {
            Stmt::Let { name, value } => {
                assert_eq!(name, "score");
                assert_eq!(value, Expr::Number(100.0));
            }
            _ => panic!("Expected Stmt::Let"),
        }
    }

    #[test]
    fn test_expression_statement() {
        let tokens = vec![
            Token::Number(10.0),
            Token::Minus,
            Token::Number(4.0),
            Token::Semicolon,
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.statement().unwrap();

        match result {
            Stmt::ExprStmt(Expr::Binary { op, left, right }) => {
                assert_eq!(op, BinOp::Sub);
                assert_eq!(*left, Expr::Number(10.0));
                assert_eq!(*right, Expr::Number(4.0));
            }
            _ => panic!("Expected Stmt::ExprStmt with Binary Expr"),
        }
    }

    #[test]
    fn test_invalid_let_statement() {
        let tokens = vec![
            Token::Let,
            Token::Number(5.0),
            Token::Equals,
            Token::Number(5.0),
            Token::Semicolon,
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);
        assert!(parser.statement().is_err());
    }

    #[test]
    fn test_comparison_precedence() {
        let tokens = vec![
            Token::Number(5.0),
            Token::Plus,
            Token::Number(3.0),
            Token::StrictEquals,
            Token::Number(8.0),
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);
        let result = parser.expression().unwrap();

        let expected = Expr::Binary {
            op: BinOp::StrictEq,
            left: Box::new(Expr::Binary {
                op: BinOp::Add,
                left: Box::new(Expr::Number(5.0)),
                right: Box::new(Expr::Number(3.0)),
            }),
            right: Box::new(Expr::Number(8.0)),
        };

        assert_eq!(result, expected);
    }
}
