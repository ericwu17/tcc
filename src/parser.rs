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
pub enum Expr {
    UnOp(UnOp, Box<Expr>),
    Int(i32),
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

    expr = generate_expr_ast(tokens);

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

pub fn generate_expr_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Expr {
    match tokens.next() {
        Some(Token::IntLit { val: v }) => {
            return Expr::Int(i32::from_str_radix(&v, 10).unwrap());
        }
        Some(Token::Op(op)) if op.to_un_op().is_some() => {
            let inner_expr = generate_expr_ast(tokens);
            return Expr::UnOp(op.to_un_op().unwrap(), Box::new(inner_expr));
        }
        _ => {
            panic!()
        }
    }
}
