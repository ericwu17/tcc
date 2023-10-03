use crate::tokenizer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub statement: Statement,
}

#[derive(Debug)]
pub enum Statement {
    // the only type of statement we know is the return statement
    Return(Expr),
}

#[derive(Debug)]
pub enum UnOp {
    Negation,
    BitwiseComplement,
    Not,
}

#[derive(Debug)]
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

#[derive(Clone, Copy, Debug)]
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
    UnOp(UnOp, Box<Expr>),
    BinOp(BinOp, Box<Expr>, Box<Expr>),
}

pub fn generate_program_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Program {
    let f = generate_function_ast(tokens);
    Program { function: f }
}

pub fn generate_function_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Function {
    let function_name;
    let statement;

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

    statement = generate_statement_ast(tokens);

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
        statement,
    }
}

pub fn generate_statement_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Statement {
    let expr;

    match tokens.peek() {
        Some(Token::Return) => {
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

    match tokens.peek() {
        Some(Token::Semicolon) => {
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    Statement::Return(expr)
}

pub fn generate_expr_ast(
    tokens: &mut Peekable<IntoIter<Token>>,
    curr_operator_precedence: BinOpPrecedenceLevel,
) -> Expr {
    let mut expr: Expr;
    let next_operator_precedence_option = curr_operator_precedence.next_level();

    if let Some(next_operator_precedence) = next_operator_precedence_option {
        expr = generate_expr_ast(tokens, next_operator_precedence);
    } else {
        expr = generate_factor_ast(tokens);
    }

    while tokens.peek().is_some() {
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

pub fn generate_factor_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Expr {
    match tokens.peek() {
        Some(Token::OpenParen) => {
            tokens.next();

            let expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

            assert!(tokens.next() == Some(Token::CloseParen));
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
        _ => panic!(),
    }
}
