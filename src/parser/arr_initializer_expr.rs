use crate::{
    errors::display::err_display,
    parser::expr_parser::{generate_expr_ast, BinOpPrecedenceLevel},
    tokenizer::Token,
    types::VarType,
};

use super::{
    expr_parser::{Expr, ExprEnum},
    token_cursor::TokenCursor,
};

pub fn generate_arr_init_expr_ast(tokens: &mut TokenCursor, expected_type: &VarType) -> Expr {
    let mut exprs = Vec::new();

    let (max_num_elems, inner_expected_type) = match expected_type {
        VarType::Arr(a, b) => (*b, a),
        VarType::Fund(_) | VarType::Ptr(_) => err_display(
            "array initializer expression nested too deep",
            tokens.get_last_ptr(),
        ),
    };

    if tokens.next() != Some(&Token::OpenBrace) {
        err_display(
            format!("expected '{{', found {:?}", tokens.last()),
            tokens.get_last_ptr(),
        );
    }

    while tokens.peek() != Some(&Token::CloseBrace) {
        if tokens.peek() == Some(&Token::OpenBrace) {
            exprs.push(generate_arr_init_expr_ast(tokens, inner_expected_type));
        } else {
            exprs.push(generate_expr_ast(
                tokens,
                BinOpPrecedenceLevel::lowest_level(),
            ));
        }

        if tokens.peek() == Some(&Token::Comma) {
            tokens.next(); // consume the comma
        }
    }

    if tokens.next() != Some(&Token::CloseBrace) {
        err_display(
            format!("expected '{{', found {:?}", tokens.last()),
            tokens.get_last_ptr(),
        );
    }

    if exprs.len() > max_num_elems {
        err_display(
            format!(
                "too many items in initializer expression (maximum {} found {})",
                max_num_elems,
                exprs.len()
            ),
            tokens.get_last_ptr(),
        )
    }

    Expr::new(ExprEnum::ArrInitExpr(exprs))
}
