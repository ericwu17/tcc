use crate::parser::expr_parser::{generate_expr_ast, BinOpPrecedenceLevel};
use crate::parser::{generate_statement_ast, Statement, TokenCursor};
use crate::tokenizer::Token;

pub fn generate_for_loop_ast(tokens: &mut TokenCursor) -> Statement {
    assert_eq!(tokens.next(), Some(&Token::For));
    assert_eq!(tokens.next(), Some(&Token::OpenParen));

    let initial_clause;
    let controlling_expr;
    let post_expr;
    let loop_body;

    if tokens.peek() == Some(&Token::IntT) {
        // initial clause is a declare statement
        initial_clause = generate_for_loop_decl_expr(tokens);
    } else if tokens.peek() == Some(&Token::Semicolon) {
        initial_clause = Statement::Empty;
    } else {
        initial_clause = Statement::Expr(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ));
    }

    assert_eq!(tokens.next(), Some(&Token::Semicolon));

    if tokens.peek() == Some(&Token::Semicolon) {
        controlling_expr = Statement::Empty;
    } else {
        controlling_expr = Statement::Expr(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ));
    }

    assert_eq!(tokens.next(), Some(&Token::Semicolon));

    if tokens.peek() == Some(&Token::CloseParen) {
        post_expr = Statement::Empty;
    } else {
        post_expr = Statement::Expr(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ));
    }

    assert_eq!(tokens.next(), Some(&Token::CloseParen));

    loop_body = generate_statement_ast(tokens);

    return Statement::For(
        Box::new(initial_clause),
        Box::new(controlling_expr),
        Box::new(post_expr),
        Box::new(loop_body),
    );
}

fn generate_for_loop_decl_expr(tokens: &mut TokenCursor) -> Statement {
    // consume the "int"
    tokens.next();
    let decl_identifier;
    if let Some(Token::Identifier { val }) = tokens.next() {
        decl_identifier = val.clone();
    } else {
        panic!();
    }

    assert_eq!(tokens.next(), Some(&Token::AssignmentEquals));

    let expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

    return Statement::Declare(decl_identifier, Some(expr));
}
