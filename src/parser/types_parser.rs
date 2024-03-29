use std::collections::VecDeque;

use crate::{
    errors::display::err_display,
    tokenizer::{operator::Op, source_cursor::SourcePtr, Token},
    types::VarType,
};

use super::{
    arr_initializer_expr::generate_arr_init_expr_ast,
    expr_parser::{generate_expr_ast, BinOpPrecedenceLevel},
    token_cursor::TokenCursor,
    Statement,
};

pub fn parse_variable_declaration(tokens: &mut TokenCursor) -> Statement {
    let fund_t;
    if let Some(Token::Type(t)) = tokens.next() {
        fund_t = VarType::Fund(*t);
    } else {
        err_display("expected fundamental type first", tokens.get_last_ptr());
    }

    let mut token_buffer = VecDeque::new();
    while tokens.peek().is_some()
        && tokens.peek().unwrap() != &Token::Semicolon
        && tokens.peek().unwrap() != &Token::Op(Op::AssignmentEquals)
    {
        token_buffer.push_back(tokens.next().unwrap().clone());
    }

    let (decl_identifier, type_) =
        parse_type_declaration(token_buffer, tokens.get_last_ptr(), fund_t);

    let mut optional_expr = None;
    if tokens.peek() == Some(&Token::Op(Op::AssignmentEquals)) {
        tokens.next(); // consume the '='
        match type_ {
            VarType::Fund(_) | VarType::Ptr(_) => {
                optional_expr = Some(generate_expr_ast(
                    tokens,
                    BinOpPrecedenceLevel::lowest_level(),
                ))
            }
            VarType::Arr(_, _) => optional_expr = Some(generate_arr_init_expr_ast(tokens, &type_)),
        }
    }
    Statement::Declare(decl_identifier, optional_expr, type_)
}

fn parse_type_declaration(
    mut tokens: VecDeque<Token>,
    location: SourcePtr,
    fund_t: VarType,
) -> (String, VarType) {
    // since the type syntax in C must be read from inside out, we will read a bunch of
    // tokens into a VecDeque and then peel off from both ends,
    // prioritizing pointers before arrays

    let mut type_ = fund_t;

    loop {
        if tokens.is_empty() {
            err_display("expected identifier name", location)
        } else if tokens.len() == 1 {
            match tokens.pop_back() {
                Some(Token::Identifier { val }) => return (val.clone(), type_),
                _ => err_display("expected identifier name", location),
            }
        } else if tokens.front() == Some(&Token::OpenParen)
            && tokens.back() == Some(&Token::CloseParen)
        {
            tokens.pop_back();
            tokens.pop_front();
        } else if tokens.front() == Some(&Token::Star) {
            tokens.pop_front();
            type_ = VarType::Ptr(Box::new(type_));
        } else if tokens.back() == Some(&Token::CloseBracket) {
            tokens.pop_back();

            let arr_size: usize;
            if let Some(Token::IntLit { val }) = tokens.pop_back() {
                arr_size = str::parse(&val).unwrap();
                if arr_size == 0 {
                    err_display("error parsing array type: zero length", location)
                }
            } else {
                err_display(
                    "error parsing array type, integer length not found",
                    location,
                )
            }

            if let Some(Token::OpenBracket) = tokens.pop_back() {
                // do nothing, just consume the close bracket
            } else {
                err_display("error parsing array type, OpenBracket not found", location)
            }

            type_ = VarType::Arr(Box::new(type_), arr_size);
        } else {
            err_display("error parsing declaration type", location);
        }
    }
}
