use crate::tokenizer::Token;
use std::iter::Peekable;
use std::vec::IntoIter;

#[derive(Debug)]
pub struct Program {
    function: Function,
}

#[derive(Debug)]
pub struct Function {
    name: String,
    statement: Statement,
}

#[derive(Debug)]
pub struct Statement {
    expr: i32,
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

    match tokens.peek() {
        Some(Token::IntExpr { val }) => {
            match i32::from_str_radix(val, 10) {
                Ok(v) => {
                    expr = v;
                    tokens.next();
                }
                Err(_) => {
                    panic!()
                }
            };
        }
        _ => {
            panic!()
        }
    }

    match tokens.peek() {
        Some(Token::Semicolon) => {
            tokens.next();
        }
        _ => {
            panic!()
        }
    }

    Statement { expr }
}
