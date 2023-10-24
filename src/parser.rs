pub mod expr_parser;
pub mod for_loop_parser;
pub mod token_cursor;
pub mod types_parser;

use crate::errors::display::err_display;
use crate::parser::{expr_parser::generate_expr_ast, token_cursor::TokenCursor};
use crate::tokenizer::source_cursor::SourcePtr;
use crate::tokenizer::Token;
use crate::types::VarType;
use expr_parser::{BinOpPrecedenceLevel, Expr};
use for_loop_parser::generate_for_loop_ast;

use self::types_parser::parse_variable_declaration;

#[derive(Debug)]
pub struct Program {
    pub functions: Vec<Function>,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub args: Vec<(String, VarType)>,
    pub body: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Continue,
    Break,
    Return(Expr),
    Declare(String, Option<Expr>, VarType),
    CompoundStmt(Vec<Statement>),
    If(Expr, Box<Statement>, Option<Box<Statement>>),
    While(Expr, Box<Statement>),
    For(Box<Statement>, Option<Expr>, Option<Expr>, Box<Statement>),
    Expr(Expr),
    Empty,
}

pub fn generate_program_ast(tokens: Vec<(Token, SourcePtr)>) -> Program {
    let mut tokens = TokenCursor::new(tokens);

    let mut functions = Vec::new();
    while tokens.peek().is_some() {
        let f = generate_function_ast(&mut tokens);
        functions.push(f);
    }

    Program { functions }
}

fn generate_function_ast(tokens: &mut TokenCursor) -> Function {
    let function_name;

    match tokens.next() {
        Some(&Token::Type(..)) => {
            // ok
        }
        _ => {
            err_display(
                "function definitions must begin with a type that they return!",
                tokens.get_last_ptr(),
            );
        }
    }

    if let Some(Token::Identifier { val }) = tokens.next() {
        function_name = val.clone();
    } else {
        err_display(
            "function name must be an identifier!",
            tokens.get_last_ptr(),
        );
    }

    if tokens.next() != Some(&Token::OpenParen) {
        err_display(
            "expected `(` to begin function arguments!",
            tokens.get_last_ptr(),
        )
    }

    let function_args = parse_function_arg_decl(tokens);

    if tokens.next() != Some(&Token::CloseParen) {
        err_display(
            "expected `)` to end function arguments!",
            tokens.get_last_ptr(),
        )
    }

    let body = generate_compound_stmt_ast(tokens);

    Function {
        name: function_name,
        args: function_args,
        body,
    }
}

fn parse_function_arg_decl(tokens: &mut TokenCursor) -> Vec<(String, VarType)> {
    // TODO: support functions arguments of type pointer to something.
    let mut args = Vec::new();

    if tokens.peek() == Some(&Token::CloseParen) {
        return Vec::new();
    }
    loop {
        let arg_type;
        let arg_name;
        if let Some(Token::Type(t)) = tokens.next() {
            arg_type = VarType::Fund(*t);
        } else {
            err_display(
                format!(
                    "expected type of argument to be specified, found {:?}",
                    tokens.last()
                ),
                tokens.get_last_ptr(),
            )
        }
        if let Some(Token::Identifier { val }) = tokens.next() {
            arg_name = val;
        } else {
            err_display(
                format!(
                    "expected identifier for function argument, found {:?}",
                    tokens.last()
                ),
                tokens.get_last_ptr(),
            )
        }
        args.push((arg_name.clone(), arg_type));

        if tokens.peek() == Some(&Token::Comma) {
            tokens.next(); // consume the comma
        } else {
            break;
        }
    }

    args
}

fn generate_compound_stmt_ast(tokens: &mut TokenCursor) -> Vec<Statement> {
    if tokens.next() != Some(&Token::OpenBrace) {
        err_display(
            "expected compound statement to begin with '{'",
            tokens.get_last_ptr(),
        );
    }
    let mut statements = Vec::new();

    while tokens.peek().is_some() && *tokens.peek().unwrap() != Token::CloseBrace {
        statements.push(generate_statement_ast(tokens));
    }

    if tokens.next() != Some(&Token::CloseBrace) {
        err_display(
            "expected compound statement to end with '}'",
            tokens.get_last_ptr(),
        );
    }
    return statements;
}

fn generate_statement_ast(tokens: &mut TokenCursor) -> Statement {
    let expr;
    let stmt;
    let mut expect_trailing_semicolon = true;

    match tokens.peek() {
        Some(Token::Continue) => {
            tokens.next(); // consume the return
            stmt = Statement::Continue;
        }
        Some(Token::Break) => {
            tokens.next(); // consume the "break"
            stmt = Statement::Break;
        }
        Some(Token::Return) => {
            tokens.next(); // consume the "return"
            expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
            stmt = Statement::Return(expr);
        }
        Some(Token::Type(_)) => {
            stmt = parse_variable_declaration(tokens);
        }
        Some(Token::OpenBrace) => {
            let compound_stmt = generate_compound_stmt_ast(tokens);
            // note that a compound statement does not end in a semicolon, so there is no need here to consume a semicolon.
            expect_trailing_semicolon = false;
            stmt = Statement::CompoundStmt(compound_stmt);
        }
        Some(Token::If) => {
            tokens.next(); // consume the "if"
            if tokens.next() != Some(&Token::OpenParen) {
                err_display("expected open paren", tokens.get_last_ptr());
            }
            let conditional_expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
            if tokens.next() != Some(&Token::CloseParen) {
                err_display("expected close paren", tokens.get_last_ptr());
            }
            let taken_branch_stmt = generate_statement_ast(tokens);
            let mut not_taken_branch_stmt = None;
            if tokens.peek() == Some(&Token::Else) {
                // consume the "else"
                tokens.next();
                not_taken_branch_stmt = Some(Box::new(generate_statement_ast(tokens)));
            }

            expect_trailing_semicolon = false;
            stmt = Statement::If(
                conditional_expr,
                Box::new(taken_branch_stmt),
                not_taken_branch_stmt,
            );
        }
        Some(Token::While) => {
            // consume the "while"
            tokens.next();

            if tokens.next() != Some(&Token::OpenParen) {
                err_display("expected open paren", tokens.get_last_ptr());
            }
            let conditional = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
            if tokens.next() != Some(&Token::CloseParen) {
                err_display("expected close paren", tokens.get_last_ptr());
            }

            let body = generate_statement_ast(tokens);

            expect_trailing_semicolon = false;
            stmt = Statement::While(conditional, Box::new(body));
        }
        Some(Token::Semicolon) => {
            stmt = Statement::Empty;
        }
        Some(Token::For) => {
            expect_trailing_semicolon = false;
            stmt = generate_for_loop_ast(tokens);
        }

        _ => {
            expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
            stmt = Statement::Expr(expr);
        }
    }

    if expect_trailing_semicolon {
        if tokens.next() != Some(&Token::Semicolon) {
            err_display("expected semicolon after statement", tokens.get_last_ptr())
        }
    }

    return stmt;
}
