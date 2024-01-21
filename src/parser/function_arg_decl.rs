use std::collections::VecDeque;

use crate::{
    errors::display::err_display,
    tokenizer::{source_cursor::SourcePtr, Token},
    types::VarType,
};

use super::token_cursor::TokenCursor;

pub fn parse_function_arg_decl(tokens: &mut TokenCursor) -> Vec<(String, VarType)> {
    let mut args = Vec::new();

    if tokens.peek() == Some(&Token::CloseParen) {
        return Vec::new();
    }

    loop {
        let mut token_buffer = VecDeque::new();
        let mut paren_level = 0;
        while tokens.peek().is_some()
            && tokens.peek().unwrap() != &Token::Comma
            && (paren_level > 0 || tokens.peek().unwrap() != &Token::CloseParen)
        {
            let next_token = tokens.next().unwrap().clone();
            if next_token == Token::OpenParen {
                paren_level += 1;
            } else if next_token == Token::CloseParen {
                paren_level -= 1;
            }
            token_buffer.push_back(next_token);
        }
        let arg = parse_type_declaration(token_buffer, tokens.get_last_ptr());
        args.push(arg);
        if tokens.peek() == Some(&Token::Comma) {
            tokens.next(); // consume the comma
        } else {
            break;
        }
    }
    args
}

fn parse_type_declaration(mut tokens: VecDeque<Token>, location: SourcePtr) -> (String, VarType) {
    let mut type_;
    if let Some(Token::Type(t)) = tokens.front() {
        type_ = VarType::Fund(*t);
        tokens.pop_front();
    } else {
        err_display(
            format!(
                "expected type of argument to be specified, found {:?}",
                tokens.front()
            ),
            location,
        )
    }

    loop {
        if tokens.is_empty() {
            err_display("expected identifier name", location)
        } else if tokens.len() == 1 {
            match tokens.pop_back() {
                Some(Token::Identifier { val }) => {
                    if let VarType::Arr(inner, _) = type_ {
                        type_ = VarType::Ptr(inner);
                    }
                    return (val.clone(), type_);
                }
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
            if let Some(Token::IntLit { val }) = tokens.back() {
                arr_size = str::parse(val).unwrap();
                tokens.pop_back();

                if let Some(Token::OpenBracket) = tokens.pop_back() {
                    // do nothing, just consume the close bracket
                } else {
                    err_display("error parsing array type, OpenBracket not found", location)
                }
                type_ = VarType::Arr(Box::new(type_), arr_size);
            } else {
                if let Some(Token::OpenBracket) = tokens.pop_back() {
                    // do nothing, just consume the close bracket
                } else {
                    err_display("error parsing array type, OpenBracket not found", location)
                }
                type_ = VarType::Ptr(Box::new(type_));
            }
        } else {
            dbg!(type_);
            dbg!(tokens);
            err_display("error parsing declaration type", location);
        }
    }
}
