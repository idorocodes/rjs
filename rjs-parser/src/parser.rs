use crate::lexer::Token::{self};

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
        match self.peek() {
            Token::Let => {
                self.advance();

                let name = match self.advance() {
                    Token::Identifier(id) => id,
                    other => {
                        return Err(ParseError {
                            message: format!(
                                "Expected variable name after 'let', found {:?}",
                                other
                            ),
                        });
                    }
                };

                self.expect(Token::Equals)?;
                let value = self.expression()?;
                self.expect(Token::Semicolon)?;

                Ok(Stmt::Let { name, value })
            }
            Token::If => self.if_statement(),
            Token::While => self.while_statement(),
            Token::For => self.for_statement(),
            Token::LeftBrace => Ok(Stmt::Block(self.block()?)),
            Token::Return => self.return_statement(),
            Token::Function => self.function_declr(),
            Token::Break => self.breakfn(),
            Token::Continue => self.continuefn(),
            _ => {
                let expr = self.expression()?;
                self.expect(Token::Semicolon)?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    pub fn program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut stmts: Vec<Stmt> = Vec::new();

        while !self.check(Token::Eof) {
            stmts.push(self.statement()?);
        }

        Ok(stmts)
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
    pub fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance(); // Consume 'return'

        if self.check(Token::Semicolon) {
            self.advance(); // Consume ';'
            Ok(Stmt::Return(None))
        } else {
            let expr = self.expression()?;
            self.expect(Token::Semicolon)?;
            Ok(Stmt::Return(Some(expr)))
        }
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
    pub fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.advance();

        self.expect(Token::LeftParen)?;

        let init = if self.check(Token::Semicolon) {
            self.advance();
            None
        } else {
            Some(Box::new(self.statement()?))
        };

        let cond = if self.check(Token::Semicolon) {
            self.advance(); // Consume 2nd ';'
            None
        } else {
            let expr = self.expression()?;
            self.expect(Token::Semicolon)?;
            Some(expr)
        };

        let update = if self.check(Token::RightParen) {
            None
        } else {
            Some(self.expression()?)
        };

        self.expect(Token::RightParen)?;
        let body = self.statement()?;

        Ok(Stmt::For {
            init,
            cond,
            update,
            body: Box::new(body),
        })
    }

    pub fn breakfn(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        self.expect(Token::Semicolon)?;

        Ok(Stmt::Break)
    }
    pub fn continuefn(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        self.expect(Token::Semicolon)?;

        Ok(Stmt::Continue)
    }

    pub fn function_declr(&mut self) -> Result<Stmt, ParseError> {
        self.advance();
        let name = match self.advance() {
            Token::Identifier(id) => id,
            other => {
                return Err(ParseError {
                    message: format!("Expected function name, found {:?}", other),
                });
            }
        };

        self.expect(Token::LeftParen)?;
        let mut params = Vec::new();

        if !self.check(Token::RightParen) {
            match self.advance() {
                Token::Identifier(id) => params.push(id),
                other => {
                    return Err(ParseError {
                        message: format!(
                            "Expected identifier in parameter list, found {:?}",
                            other
                        ),
                    });
                }
            }

            while self.check(Token::Comma) {
                self.advance(); // Consume the comma
                match self.advance() {
                    Token::Identifier(id) => params.push(id),
                    other => {
                        return Err(ParseError {
                            message: format!("Expected identifier after comma, found {:?}", other),
                        });
                    }
                }
            }
        }

        self.expect(Token::RightParen)?;

        let body = self.block()?;
        Ok(Stmt::FunctionDecl { name, params, body })
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

#[derive(Debug, Clone, PartialEq)]
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

    fn parse_stmts(tokens: Vec<Token>) -> Result<Vec<Stmt>, ParseError> {
        let mut parser = Parser::new(tokens);
        parser.program()
    }

    #[test]
    fn test_unexpected_primary_token() {
        let tokens = vec![Token::Plus, Token::Eof];
        let mut parser = Parser::new(tokens);
        assert!(parser.primary().is_err());
    }

    #[test]
    fn test_operator_precedence() {
        // 1 + 2 * 3 == 7 && !false
        let tokens = vec![
            Token::Number(1.0),
            Token::Plus,
            Token::Number(2.0),
            Token::Star,
            Token::Number(3.0),
            Token::EqualsEquals,
            Token::Number(7.0),
            Token::AndAnd,
            Token::Bang,
            Token::False,
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);

        let expected = Expr::Binary {
            op: BinOp::And,
            left: Box::new(Expr::Binary {
                op: BinOp::Eq,
                left: Box::new(Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(Expr::Number(1.0)),
                    right: Box::new(Expr::Binary {
                        op: BinOp::Mul,
                        left: Box::new(Expr::Number(2.0)),
                        right: Box::new(Expr::Number(3.0)),
                    }),
                }),
                right: Box::new(Expr::Number(7.0)),
            }),
            right: Box::new(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Boolean(false)),
            }),
        };

        assert_eq!(parser.expression().unwrap(), expected);
    }

    #[test]
    fn test_postfix_operations() {
        // obj.prop[0](a, b)
        let tokens = vec![
            Token::Identifier("obj".to_string()),
            Token::Dot,
            Token::Identifier("prop".to_string()),
            Token::LeftBracket,
            Token::Number(0.0),
            Token::RightBracket,
            Token::LeftParen,
            Token::Identifier("a".to_string()),
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::RightParen,
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);

        let expected = Expr::Call {
            callee: Box::new(Expr::Index {
                object: Box::new(Expr::Member {
                    object: Box::new(Expr::Identifier("obj".to_string())),
                    property: "prop".to_string(),
                }),
                index: Box::new(Expr::Number(0.0)),
            }),
            args: vec![
                Expr::Identifier("a".to_string()),
                Expr::Identifier("b".to_string()),
            ],
        };

        assert_eq!(parser.expression().unwrap(), expected);
    }

    #[test]
    fn test_assignment_right_associativity() {
        // x = y = 5
        let tokens = vec![
            Token::Identifier("x".to_string()),
            Token::Equals,
            Token::Identifier("y".to_string()),
            Token::Equals,
            Token::Number(5.0),
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);

        let expected = Expr::Assign {
            name: "x".to_string(),
            value: Box::new(Expr::Assign {
                name: "y".to_string(),
                value: Box::new(Expr::Number(5.0)),
            }),
        };

        assert_eq!(parser.expression().unwrap(), expected);
    }

    #[test]
    fn test_invalid_assignment_target() {
        // 5 = x
        let tokens = vec![
            Token::Number(5.0),
            Token::Equals,
            Token::Identifier("x".to_string()),
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);
        assert!(parser.expression().is_err());
    }

    #[test]
    fn test_if_else_statement() {
        // if (x) { return true; } else { return false; }
        let tokens = vec![
            Token::If,
            Token::LeftParen,
            Token::Identifier("x".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::True,
            Token::Semicolon,
            Token::RightBrace,
            Token::Else,
            Token::LeftBrace,
            Token::Return,
            Token::False,
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        let expected = vec![Stmt::If {
            cond: Expr::Identifier("x".to_string()),
            then_branch: Box::new(Stmt::Block(vec![Stmt::Return(Some(Expr::Boolean(true)))])),
            else_branch: Some(Box::new(Stmt::Block(vec![Stmt::Return(Some(
                Expr::Boolean(false),
            ))]))),
        }];

        assert_eq!(parse_stmts(tokens).unwrap(), expected);
    }

    #[test]
    fn test_while_loop() {
        // while (running) { break; }
        let tokens = vec![
            Token::While,
            Token::LeftParen,
            Token::Identifier("running".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Break,
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        let mut parser = Parser::new(tokens);
        let stmt = parser.statement().unwrap();

        let expected = Stmt::While {
            cond: Expr::Identifier("running".to_string()),
            body: Box::new(Stmt::Block(vec![Stmt::Break])),
        };

        assert_eq!(stmt, expected);
    }

    #[test]
    fn test_for_loop() {
        // for (let i = 0; i < 10; i = i + 1) { continue; }
        let tokens = vec![
            Token::For,
            Token::LeftParen,
            Token::Let,
            Token::Identifier("i".to_string()),
            Token::Equals,
            Token::Number(0.0),
            Token::Semicolon,
            Token::Identifier("i".to_string()),
            Token::Less,
            Token::Number(10.0),
            Token::Semicolon,
            Token::Identifier("i".to_string()),
            Token::Equals,
            Token::Identifier("i".to_string()),
            Token::Plus,
            Token::Number(1.0),
            Token::RightParen,
            Token::LeftBrace,
            Token::Continue,
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        let expected = vec![Stmt::For {
            init: Some(Box::new(Stmt::Let {
                name: "i".to_string(),
                value: Expr::Number(0.0),
            })),
            cond: Some(Expr::Binary {
                op: BinOp::Lt,
                left: Box::new(Expr::Identifier("i".to_string())),
                right: Box::new(Expr::Number(10.0)),
            }),
            update: Some(Expr::Assign {
                name: "i".to_string(),
                value: Box::new(Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(Expr::Identifier("i".to_string())),
                    right: Box::new(Expr::Number(1.0)),
                }),
            }),
            body: Box::new(Stmt::Block(vec![Stmt::Continue])),
        }];

        assert_eq!(parse_stmts(tokens).unwrap(), expected);
    }

    #[test]
    fn test_function_declaration() {
        // function add(a, b) { return a + b; }
        let tokens = vec![
            Token::Function,
            Token::Identifier("add".to_string()),
            Token::LeftParen,
            Token::Identifier("a".to_string()),
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::RightParen,
            Token::LeftBrace,
            Token::Return,
            Token::Identifier("a".to_string()),
            Token::Plus,
            Token::Identifier("b".to_string()),
            Token::Semicolon,
            Token::RightBrace,
            Token::Eof,
        ];

        let expected = vec![Stmt::FunctionDecl {
            name: "add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            body: vec![Stmt::Return(Some(Expr::Binary {
                op: BinOp::Add,
                left: Box::new(Expr::Identifier("a".to_string())),
                right: Box::new(Expr::Identifier("b".to_string())),
            }))],
        }];

        assert_eq!(parse_stmts(tokens).unwrap(), expected);
    }

    // --- ERROR HANDLING TESTS ---

    #[test]
    fn test_missing_semicolon() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("a".to_string()),
            Token::Equals,
            Token::Number(1.0),
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);
        assert!(parser.statement().is_err());
    }

    #[test]
    fn test_invalid_function_param() {
        let tokens = vec![
            Token::Function,
            Token::Identifier("foo".to_string()),
            Token::LeftParen,
            Token::Number(123.0),
            Token::RightParen,
            Token::LeftBrace,
            Token::RightBrace,
            Token::Eof,
        ];
        let mut parser = Parser::new(tokens);
        assert!(parser.statement().is_err());
    }
}
