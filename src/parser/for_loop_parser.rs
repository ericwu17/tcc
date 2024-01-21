use super::types_parser::parse_variable_declaration;
use crate::errors::display::err_display;
use crate::parser::expr_parser::{generate_expr_ast, BinOpPrecedenceLevel};
use crate::parser::{generate_statement_ast, Statement, TokenCursor};
use crate::tokenizer::Token;

pub fn generate_for_loop_ast(tokens: &mut TokenCursor) -> Statement {
    assert_eq!(tokens.next(), Some(&Token::For)); // should be true because this function is only called when we need to parse a for loop (caller should have peeked)
    if tokens.next() != Some(&Token::OpenParen) {
        err_display(
            format!(
                "expected opening parenthesis for for loop, found {:?}",
                tokens.last().unwrap()
            ),
            tokens.get_last_ptr(),
        )
    }

    let initial_clause = if let Some(&Token::Type(_)) = tokens.peek() {
        // initial clause is a declare statement
        parse_variable_declaration(tokens)
    } else if tokens.peek() == Some(&Token::Semicolon) {
        Statement::Empty
    } else {
        Statement::Expr(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ))
    };

    if tokens.next() != Some(&Token::Semicolon) {
        err_display(
            format!(
                "expected semicolon for for loop, found {:?}",
                tokens.last().unwrap()
            ),
            tokens.get_last_ptr(),
        )
    }

    let controlling_expr = if tokens.peek() == Some(&Token::Semicolon) {
        None
    } else {
        Some(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ))
    };

    if tokens.next() != Some(&Token::Semicolon) {
        err_display(
            format!(
                "expected semicolon for for loop, found {:?}",
                tokens.last().unwrap()
            ),
            tokens.get_last_ptr(),
        )
    }

    let post_expr = if tokens.peek() == Some(&Token::CloseParen) {
        None
    } else {
        Some(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ))
    };

    if tokens.next() != Some(&Token::CloseParen) {
        err_display(
            format!(
                "expected closing parenthesis for for loop, found {:?}",
                tokens.last().unwrap()
            ),
            tokens.get_last_ptr(),
        )
    }

    let loop_body = generate_statement_ast(tokens);

    Statement::For(
        Box::new(initial_clause),
        controlling_expr,
        post_expr,
        Box::new(loop_body),
    )
}
