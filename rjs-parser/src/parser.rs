use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct Parser {
    tokens : Vec<Token>,
    pos : usize,
}

pub struct ParseError{
    message : String
}

impl Parser{
    pub fn new (tokens : Vec<Token> ) -> Parser {
        Self{
            tokens,
            pos : 0
        }
    }


    pub  fn peek(&self) -> & Token{
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
    pub fn check(&self,token:Token) -> bool {
        *self.peek() == token
    }


    pub fn expect(&mut self, token:Token) -> Result<(),ParseError>{
        if self.check(token.clone()) {
            self.advance();
            Ok(())
        }else {
            return Err(ParseError { message: format!("Unable to parse token: {:?}",token) });
        }


    }




}
#[derive(Debug, Clone)]
pub enum Expr {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Identifier(String),

    Unary { op: UnaryOp, operand: Box<Expr> },
    Binary { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    Assign { name: String, value: Box<Expr> },

    Call { callee: Box<Expr>, args: Vec<Expr> },
    Member { object: Box<Expr>, property: String },   // user.name
    Index { object: Box<Expr>, index: Box<Expr> },    // nums[i]

    ObjectLiteral(Vec<(String, Expr)>),
    ArrayLiteral(Vec<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: String, value: Expr },
    Const { name: String, value: Expr },
    ExprStmt(Expr),
    Block(Vec<Stmt>),

    If {
        cond: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While { cond: Expr, body: Box<Stmt> },
    For {
        init: Option<Box<Stmt>>,
        cond: Option<Expr>,
        update: Option<Expr>,
        body: Box<Stmt>,
    },

    Return(Option<Expr>),
    Break,
    Continue,
    FunctionDecl { name: String, params: Vec<String>, body: Vec<Stmt> },
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, // -x
    Not, // !x
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div,
    Eq, StrictEq, NotEq,
    Lt, Gt, LtEq, GtEq,
    And, Or,
}



