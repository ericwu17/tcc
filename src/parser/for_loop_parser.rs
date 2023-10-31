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

    let initial_clause;
    let controlling_expr;
    let post_expr;
    let loop_body;

    if let Some(&Token::Type(_)) = tokens.peek() {
        // initial clause is a declare statement
        initial_clause = parse_variable_declaration(tokens);
    } else if tokens.peek() == Some(&Token::Semicolon) {
        initial_clause = Statement::Empty;
    } else {
        initial_clause = Statement::Expr(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ));
    }

    if tokens.next() != Some(&Token::Semicolon) {
        err_display(
            format!(
                "expected semicolon for for loop, found {:?}",
                tokens.last().unwrap()
            ),
            tokens.get_last_ptr(),
        )
    }

    if tokens.peek() == Some(&Token::Semicolon) {
        controlling_expr = None;
    } else {
        controlling_expr = Some(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ));
    }

    if tokens.next() != Some(&Token::Semicolon) {
        err_display(
            format!(
                "expected semicolon for for loop, found {:?}",
                tokens.last().unwrap()
            ),
            tokens.get_last_ptr(),
        )
    }

    if tokens.peek() == Some(&Token::CloseParen) {
        post_expr = None;
    } else {
        post_expr = Some(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ));
    }

    if tokens.next() != Some(&Token::CloseParen) {
        err_display(
            format!(
                "expected closing parenthesis for for loop, found {:?}",
                tokens.last().unwrap()
            ),
            tokens.get_last_ptr(),
        )
    }

    loop_body = generate_statement_ast(tokens);

    return Statement::For(
        Box::new(initial_clause),
        controlling_expr,
        post_expr,
        Box::new(loop_body),
    );
}
