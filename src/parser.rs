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
    Plus,
    Minus,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub enum PlusMinus {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum MulDiv {
    Multiply,
    Divide,
}

#[derive(Debug)]
pub struct Expr {
    initial_term: Term,
    remaining_terms: Vec<(PlusMinus, Term)>,
}

#[derive(Debug)]
pub struct Term {
    initial_factor: Factor,
    remaining_factors: Vec<(MulDiv, Factor)>,
}

#[derive(Debug)]
pub enum Factor {
    Expr(Box<Expr>),
    UnOp(UnOp, Box<Factor>),
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
    let term = generate_term_ast(tokens);

    let mut remaining_terms = Vec::new();

    while tokens.peek().is_some() && tokens.peek().unwrap().to_plus_minus().is_some() {
        let next_op = tokens.next().unwrap().to_plus_minus().unwrap();
        let next_term = generate_term_ast(tokens);
        remaining_terms.push((next_op, next_term))
    }

    Expr {
        initial_term: term,
        remaining_terms,
    }
}

pub fn generate_term_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Term {
    let factor = generate_factor_ast(tokens);

    let mut remaining_factors = Vec::new();

    while tokens.peek().is_some() && tokens.peek().unwrap().to_mul_div().is_some() {
        let next_op = tokens.next().unwrap().to_mul_div().unwrap();
        let next_term = generate_factor_ast(tokens);
        remaining_factors.push((next_op, next_term))
    }

    Term {
        initial_factor: factor,
        remaining_factors,
    }
}

pub fn generate_factor_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Factor {
    match tokens.peek() {
        Some(Token::OpenParen) => {
            tokens.next();

            let expr = generate_expr_ast(tokens);

            assert!(tokens.next() == Some(Token::CloseParen));

            return Factor::Expr(Box::new(expr));
        }
        Some(Token::Op(op)) if op.to_un_op().is_some() => {
            let un_op = op.to_un_op().unwrap();
            tokens.next();
            let factor = generate_factor_ast(tokens);
            return Factor::UnOp(un_op, Box::new(factor));
        }
        Some(Token::IntLit { val }) => {
            let val_i32 = i32::from_str_radix(val, 10).unwrap();
            tokens.next();

            return Factor::Int(val_i32);
        }
        _ => panic!(),
    }
}
