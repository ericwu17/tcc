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
    LogicalOr,
    LogicalAnd,
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanEq,
    LessThan,
    LessThanEq,
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
    let mut expr = generate_logical_or_term_ast(tokens);

    while tokens.peek().is_some() && tokens.peek().unwrap().to_logical_or().is_some() {
        let next_op = tokens.next().unwrap().to_logical_or().unwrap();
        let next_expr = generate_logical_or_term_ast(tokens);

        expr = Expr::BinOp(next_op, Box::new(expr), Box::new(next_expr));
    }

    expr
}

pub fn generate_logical_or_term_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Expr {
    let mut expr = generate_logical_and_term_ast(tokens);

    while tokens.peek().is_some() && tokens.peek().unwrap().to_logical_and().is_some() {
        let next_op = tokens.next().unwrap().to_logical_and().unwrap();
        let next_expr = generate_logical_and_term_ast(tokens);

        expr = Expr::BinOp(next_op, Box::new(expr), Box::new(next_expr));
    }

    expr
}

pub fn generate_logical_and_term_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Expr {
    let mut expr = generate_comparison_term_ast(tokens);

    while tokens.peek().is_some() && tokens.peek().unwrap().to_comparison_op().is_some() {
        let next_op = tokens.next().unwrap().to_comparison_op().unwrap();
        let next_expr = generate_comparison_term_ast(tokens);

        expr = Expr::BinOp(next_op, Box::new(expr), Box::new(next_expr));
    }

    expr
}

pub fn generate_comparison_term_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Expr {
    let mut expr = generate_ordering_term_ast(tokens);

    while tokens.peek().is_some() && tokens.peek().unwrap().to_ordering_op().is_some() {
        let next_op = tokens.next().unwrap().to_ordering_op().unwrap();
        let next_expr = generate_ordering_term_ast(tokens);

        expr = Expr::BinOp(next_op, Box::new(expr), Box::new(next_expr));
    }

    expr
}

pub fn generate_ordering_term_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Expr {
    let mut expr = generate_term_ast(tokens);

    while tokens.peek().is_some() && tokens.peek().unwrap().to_plus_minus().is_some() {
        let next_op = tokens.next().unwrap().to_plus_minus().unwrap();
        let next_expr = generate_term_ast(tokens);

        expr = Expr::BinOp(next_op, Box::new(expr), Box::new(next_expr));
    }

    expr
}

pub fn generate_term_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Expr {
    let mut factor = generate_factor_ast(tokens);

    while tokens.peek().is_some() && tokens.peek().unwrap().to_mul_div().is_some() {
        let next_op = tokens.next().unwrap().to_mul_div().unwrap();
        let next_factor = generate_factor_ast(tokens);
        factor = Expr::BinOp(next_op, Box::new(factor), Box::new(next_factor));
    }

    factor
}

pub fn generate_factor_ast(tokens: &mut Peekable<IntoIter<Token>>) -> Expr {
    match tokens.peek() {
        Some(Token::OpenParen) => {
            tokens.next();

            let expr = generate_expr_ast(tokens);

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
