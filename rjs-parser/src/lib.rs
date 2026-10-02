use crate::{
    lexer::tokenizer,
    parser::{ParseError, Parser, Stmt},
};

mod lexer;
mod parser;

pub fn parse(source: &str) -> Result<Vec<Stmt>, ParseError> {
    let tokens = tokenizer(source.to_string());
    let mut parser = Parser::new(tokens);
    parser.program()
}

#[cfg(test)]
mod extended_parse_integration_tests {
    use super::*;
    use crate::parser::{BinOp, Expr, Stmt, UnaryOp};

    #[test]
    fn test_parse_member_access_and_method_calls() {
        let source = "user.profile.getName();";
        let stmts = parse(source).unwrap();

        let expected = vec![Stmt::ExprStmt(Expr::Call {
            callee: Box::new(Expr::Member {
                object: Box::new(Expr::Member {
                    object: Box::new(Expr::Identifier("user".to_string())),
                    property: "profile".to_string(),
                }),
                property: "getName".to_string(),
            }),
            args: vec![],
        })];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_computed_indexing() {
        let source = "matrix[i][j + 1];";
        let stmts = parse(source).unwrap();

        let expected = vec![Stmt::ExprStmt(Expr::Index {
            object: Box::new(Expr::Index {
                object: Box::new(Expr::Identifier("matrix".to_string())),
                index: Box::new(Expr::Identifier("i".to_string())),
            }),
            index: Box::new(Expr::Binary {
                op: BinOp::Add,
                left: Box::new(Expr::Identifier("j".to_string())),
                right: Box::new(Expr::Number(1.0)),
            }),
        })];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_nested_unary_operators() {
        let source = "!!-x;";
        let stmts = parse(source).unwrap();

        let expected = vec![Stmt::ExprStmt(Expr::Unary {
            op: UnaryOp::Not,
            operand: Box::new(Expr::Unary {
                op: UnaryOp::Not,
                operand: Box::new(Expr::Unary {
                    op: UnaryOp::Neg,
                    operand: Box::new(Expr::Identifier("x".to_string())),
                }),
            }),
        })];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_logical_and_or_precedence() {
        // a || b && c should parse as a || (b && c)
        let source = "a || b && c;";
        let stmts = parse(source).unwrap();

        let expected = vec![Stmt::ExprStmt(Expr::Binary {
            op: BinOp::Or,
            left: Box::new(Expr::Identifier("a".to_string())),
            right: Box::new(Expr::Binary {
                op: BinOp::And,
                left: Box::new(Expr::Identifier("b".to_string())),
                right: Box::new(Expr::Identifier("c".to_string())),
            }),
        })];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_comparison_operators() {
        let source = "a <= b == c > d;";
        let stmts = parse(source).unwrap();

        // Left-associative comparison parsing
        let expected = vec![Stmt::ExprStmt(Expr::Binary {
            op: BinOp::Gt,
            left: Box::new(Expr::Binary {
                op: BinOp::Eq,
                left: Box::new(Expr::Binary {
                    op: BinOp::LtEq,
                    left: Box::new(Expr::Identifier("a".to_string())),
                    right: Box::new(Expr::Identifier("b".to_string())),
                }),
                right: Box::new(Expr::Identifier("c".to_string())),
            }),
            right: Box::new(Expr::Identifier("d".to_string())),
        })];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_for_loop_empty_clauses() {
        // Infinite for loop: for (;;) { break; }
        let source = "for (;;) { break; }";
        let stmts = parse(source).unwrap();

        let expected = vec![Stmt::For {
            init: None,
            cond: None,
            update: None,
            body: Box::new(Stmt::Block(vec![Stmt::Break])),
        }];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_for_loop_partial_clauses() {
        let source = "for (; i < 10;) { i = i + 1; }";
        let stmts = parse(source).unwrap();

        let expected = vec![Stmt::For {
            init: None,
            cond: Some(Expr::Binary {
                op: BinOp::Lt,
                left: Box::new(Expr::Identifier("i".to_string())),
                right: Box::new(Expr::Number(10.0)),
            }),
            update: None,
            body: Box::new(Stmt::Block(vec![Stmt::ExprStmt(Expr::Assign {
                name: "i".to_string(),
                value: Box::new(Expr::Binary {
                    op: BinOp::Add,
                    left: Box::new(Expr::Identifier("i".to_string())),
                    right: Box::new(Expr::Number(1.0)),
                }),
            })])),
        }];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_function_with_no_params_and_empty_return() {
        let source = "function doNothing() { return; }";
        let stmts = parse(source).unwrap();

        let expected = vec![Stmt::FunctionDecl {
            name: "doNothing".to_string(),
            params: vec![],
            body: vec![Stmt::Return(None)],
        }];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_nested_scopes_and_blocks() {
        let source = r#"
            let outer = 1;
            {
                let inner = 2;
                {
                    let core = 3;
                }
            }
        "#;
        let stmts = parse(source).unwrap();

        let expected = vec![
            Stmt::Let {
                name: "outer".to_string(),
                value: Expr::Number(1.0),
            },
            Stmt::Block(vec![
                Stmt::Let {
                    name: "inner".to_string(),
                    value: Expr::Number(2.0),
                },
                Stmt::Block(vec![Stmt::Let {
                    name: "core".to_string(),
                    value: Expr::Number(3.0),
                }]),
            ]),
        ];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_all_primitive_literals() {
        let source = "let a = true; let b = false; let c = null; let d = undefined;";
        let stmts = parse(source).unwrap();

        let expected = vec![
            Stmt::Let {
                name: "a".to_string(),
                value: Expr::Boolean(true),
            },
            Stmt::Let {
                name: "b".to_string(),
                value: Expr::Boolean(false),
            },
            Stmt::Let {
                name: "c".to_string(),
                value: Expr::Null,
            },
            Stmt::Let {
                name: "d".to_string(),
                value: Expr::Undefined,
            },
        ];

        assert_eq!(stmts, expected);
    }

    #[test]
    fn test_parse_error_missing_closing_paren_if() {
        let source = "if (x > 0 { return; }";
        assert!(parse(source).is_err());
    }

    #[test]
    fn test_parse_error_missing_closing_brace_block() {
        let source = "{ let x = 10; ";
        assert!(parse(source).is_err());
    }

    #[test]
    fn test_parse_error_unexpected_token_in_expression() {
        let source = "let x = + * 5;";
        assert!(parse(source).is_err());
    }

    #[test]
    fn test_parse_error_missing_property_after_dot() {
        let source = "let val = obj.;";
        assert!(parse(source).is_err());
    }

    #[test]
    fn test_parse_error_trailing_comma_or_missing_arg() {
        let source = "foo(a, );";
        assert!(parse(source).is_err());
    }
}
