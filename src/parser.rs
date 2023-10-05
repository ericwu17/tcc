use crate::tokenizer::Token;

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub statements: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Return(Expr),
    Declare(String, Option<Expr>),
    Expr(Expr),
}

#[derive(Debug)]
pub enum UnOp {
    Negation,
    BitwiseComplement,
    Not,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinOp {
    Multiply,
    Divide,
    Plus,
    Minus,
    GreaterThan,
    GreaterThanEq,
    LessThan,
    LessThanEq,
    Equals,
    NotEquals,
    LogicalAnd,
    LogicalOr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOpPrecedenceLevel {
    MulDiv,
    AddSub,
    OrderingCmp,
    EqCmp,
    LogicalAnd,
    LogicalOr,
}

impl BinOpPrecedenceLevel {
    pub fn next_level(&self) -> Option<Self> {
        match self {
            BinOpPrecedenceLevel::LogicalOr => Some(BinOpPrecedenceLevel::LogicalAnd),
            BinOpPrecedenceLevel::LogicalAnd => Some(BinOpPrecedenceLevel::EqCmp),
            BinOpPrecedenceLevel::EqCmp => Some(BinOpPrecedenceLevel::OrderingCmp),
            BinOpPrecedenceLevel::OrderingCmp => Some(BinOpPrecedenceLevel::AddSub),
            BinOpPrecedenceLevel::AddSub => Some(BinOpPrecedenceLevel::MulDiv),
            BinOpPrecedenceLevel::MulDiv => None,
        }
    }

    pub fn lowest_level() -> Self {
        BinOpPrecedenceLevel::LogicalOr
    }
}

#[derive(Debug)]
pub enum Expr {
    Int(i32),
    Var(String),
    Assign(String, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

pub struct TokenCursor {
    contents: Vec<Token>,
    index: usize,
}

impl TokenCursor {
    pub fn new(contents: Vec<Token>) -> Self {
        TokenCursor { contents, index: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.contents.get(self.index)
    }
    fn peek_nth(&self, n: usize) -> Option<&Token> {
        // peek_nth(1) is equivalent to peek()
        self.contents.get(self.index + n - 1)
    }

    fn next(&mut self) -> Option<&Token> {
        self.index += 1;
        self.contents.get(self.index - 1)
    }
}

pub fn generate_program_ast(tokens: Vec<Token>) -> Program {
    let mut tokens = TokenCursor::new(tokens);
    let f = generate_function_ast(&mut tokens);
    assert!(tokens.next() == None);
    Program { function: f }
}

pub fn generate_function_ast(tokens: &mut TokenCursor) -> Function {
    let function_name;

    match tokens.peek() {
        Some(Token::IntT) => {
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    match tokens.peek() {
        Some(Token::Identifier { val }) => {
            function_name = val.clone();
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    match tokens.peek() {
        Some(Token::OpenParen) => {
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    match tokens.peek() {
        Some(Token::CloseParen) => {
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    match tokens.peek() {
        Some(Token::OpenBrace) => {
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    let mut statements = Vec::new();

    while tokens.peek().is_some() && *tokens.peek().unwrap() != Token::CloseBrace {
        statements.push(generate_statement_ast(tokens));
    }

    match tokens.peek() {
        Some(Token::CloseBrace) => {
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    Function {
        name: function_name,
        statements,
    }
}

pub fn generate_statement_ast(tokens: &mut TokenCursor) -> Statement {
    let expr;

    match tokens.peek() {
        Some(Token::Return) => {
            tokens.next();

            expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

            assert!(tokens.next() == Some(&Token::Semicolon));
            return Statement::Return(expr);
        }
        Some(Token::IntT) => {
            tokens.next();
            let decl_identifier;
            let mut optional_expr = None;
            match tokens.next() {
                Some(Token::Identifier { val }) => {
                    decl_identifier = val.clone();
                }
                _ => {
                    panic!()
                }
            }

            if tokens.peek() == Some(&Token::AssignmentEquals) {
                tokens.next();
                optional_expr = Some(generate_expr_ast(
                    tokens,
                    BinOpPrecedenceLevel::lowest_level(),
                ))
            }
            assert!(tokens.next() == Some(&Token::Semicolon));
            return Statement::Declare(decl_identifier, optional_expr);
        }
        _ => {
            expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
            assert!(tokens.next() == Some(&Token::Semicolon));
            return Statement::Expr(expr);
        }
    }
}

pub fn generate_expr_ast(
    tokens: &mut TokenCursor,
    curr_operator_precedence: BinOpPrecedenceLevel,
) -> Expr {
    if curr_operator_precedence == BinOpPrecedenceLevel::lowest_level() {
        // handle assignment of variables
        if let Some(Token::Identifier { val }) = tokens.peek() {
            let val = val.clone();
            if tokens.peek_nth(2) == Some(&Token::AssignmentEquals) {
                tokens.next();
                tokens.next();

                let rhs_expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
                return Expr::Assign(val, Box::new(rhs_expr));
            }
        }
    }

    let mut expr: Expr;
    let next_operator_precedence_option = curr_operator_precedence.next_level();

    if let Some(next_operator_precedence) = next_operator_precedence_option {
        expr = generate_expr_ast(tokens, next_operator_precedence);
    } else {
        expr = generate_factor_ast(tokens);
    }

    while tokens.peek().is_some() {
        if &Token::QuestionMark == tokens.peek().unwrap()
            && curr_operator_precedence == BinOpPrecedenceLevel::lowest_level()
        {
            // handle ternary case. Note that ternaries have the lowest precedence level, so we need to check the precedence level.
            tokens.next();
            let first_next_expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

            assert!(tokens.next() == Some(&Token::Colon));

            let second_expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

            return Expr::Ternary(
                Box::new(expr),
                Box::new(first_next_expr),
                Box::new(second_expr),
            );
        }

        if let Some(next_op) = tokens
            .peek()
            .unwrap()
            .to_binop_precedence_level(curr_operator_precedence)
        {
            tokens.next();
            let next_expr;
            if let Some(next_operator_precedence) = next_operator_precedence_option {
                next_expr = generate_expr_ast(tokens, next_operator_precedence);
            } else {
                next_expr = generate_factor_ast(tokens);
            }
            expr = Expr::BinOp(next_op, Box::new(expr), Box::new(next_expr));
        } else {
            break;
        }
    }
    return expr;
}

pub fn generate_factor_ast(tokens: &mut TokenCursor) -> Expr {
    match tokens.peek() {
        Some(Token::OpenParen) => {
            tokens.next();

            let expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

            assert!(tokens.next() == Some(&Token::CloseParen));
            return expr;
        }
        Some(token) if token.to_un_op().is_some() => {
            let un_op = token.to_un_op().unwrap();
            tokens.next();
            let factor = generate_factor_ast(tokens);
            return Expr::UnOp(un_op, Box::new(factor));
        }
        Some(Token::IntLit { val }) => {
            let val_i32 = i32::from_str_radix(val, 10).unwrap();
            tokens.next();

            return Expr::Int(val_i32);
        }
        Some(Token::Identifier { val }) => {
            let val = val.clone();
            tokens.next();
            return Expr::Var(val);
        }
        _ => {
            dbg!(tokens.peek());
            panic!();
        }
    }
}
