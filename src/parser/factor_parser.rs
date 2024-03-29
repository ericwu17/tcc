use crate::{
    errors::display::err_display,
    tokenizer::{operator::Op, Token},
};

use super::{
    expr_parser::{generate_expr_ast, BinOp, BinOpPrecedenceLevel, Expr, ExprEnum},
    global_strings::add_static_string,
    token_cursor::TokenCursor,
};

pub fn generate_factor_ast(tokens: &mut TokenCursor) -> Expr {
    match tokens.peek() {
        Some(Token::Op(op)) if *op == Op::PlusPlus || *op == Op::MinusMinus => {
            let op = op.clone();
            tokens.next();
            let factor = generate_factor_ast(tokens);
            if op == Op::PlusPlus {
                Expr::new(ExprEnum::PrefixInc(Box::new(factor)))
            } else {
                Expr::new(ExprEnum::PrefixDec(Box::new(factor)))
            }
        }
        Some(token) if token.to_un_op().is_some() => {
            let un_op = token.to_un_op().unwrap();
            tokens.next();
            let factor = generate_factor_ast(tokens);

            Expr::new(ExprEnum::UnOp(un_op, Box::new(factor)))
        }
        Some(Token::Star) => {
            tokens.next();
            let factor = generate_factor_ast(tokens);
            Expr::new(ExprEnum::Deref(Box::new(factor)))
        }
        Some(Token::Ampersand) => {
            tokens.next();
            let factor = generate_factor_ast(tokens);
            Expr::new(ExprEnum::Ref(Box::new(factor)))
        }
        Some(Token::Identifier { val }) => {
            let val = val.clone();
            tokens.next();

            let expr = if tokens.peek() == Some(&Token::OpenParen) {
                tokens.next(); // consume the open paren
                let args = parse_function_args(tokens);
                if tokens.next() != Some(&Token::CloseParen) {
                    err_display(
                        format!(
                            "expected closing parenthesis, found {:?}",
                            tokens.last().unwrap()
                        ),
                        tokens.get_last_ptr(),
                    )
                }
                Expr::new(ExprEnum::FunctionCall(val, args))
            } else {
                Expr::new(ExprEnum::Var(val))
            };

            attach_postfix_ops(tokens, expr)
        }
        Some(Token::StringLiteral(val)) => {
            let val = val.clone();
            tokens.next();
            add_static_string(val.clone());
            Expr::new(ExprEnum::StaticStrPtr(val.clone()))
        }
        Some(Token::Sizeof) => {
            tokens.next(); // consume the "sizeof"
            assert_eq!(tokens.next(), Some(&Token::OpenParen));
            let expr = Expr::new(ExprEnum::Sizeof(Box::new(generate_expr_ast(
                tokens,
                BinOpPrecedenceLevel::lowest_level(),
            ))));
            assert_eq!(tokens.next(), Some(&Token::CloseParen));
            expr
        }

        Some(Token::IntLit { val }) => {
            let val_i32 = str::parse(val).unwrap();
            tokens.next();

            Expr::new(ExprEnum::Int(val_i32))
        }
        Some(Token::OpenParen) => {
            tokens.next(); // consume opening parenthesis

            let expr = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());

            if tokens.next() != Some(&Token::CloseParen) {
                err_display(
                    format!(
                        "expected closing parenthesis, found {:?}",
                        tokens.last().unwrap()
                    ),
                    tokens.get_last_ptr(),
                )
            }
            attach_postfix_ops(tokens, expr)
        }
        _ => err_display(
            format!("unexpected token: {:?}", tokens.peek()),
            tokens.get_last_ptr(),
        ),
    }
}

fn attach_postfix_ops(tokens: &mut TokenCursor, curr_expr: Expr) -> Expr {
    if tokens.peek() == Some(&Token::Op(Op::MinusMinus)) {
        tokens.next();
        attach_postfix_ops(tokens, Expr::new(ExprEnum::PostfixDec(Box::new(curr_expr))))
    } else if tokens.peek() == Some(&Token::Op(Op::PlusPlus)) {
        tokens.next();
        return attach_postfix_ops(tokens, Expr::new(ExprEnum::PostfixInc(Box::new(curr_expr))));
    } else if tokens.peek() == Some(&Token::OpenBracket) {
        tokens.next();
        let arr_size = generate_expr_ast(tokens, BinOpPrecedenceLevel::lowest_level());
        if tokens.next() != Some(&Token::CloseBracket) {
            err_display("expected closing bracket", tokens.get_last_ptr());
        }

        let equiv_deref_expr = Expr::new(ExprEnum::Deref(Box::new(Expr::new(ExprEnum::BinOp(
            BinOp::Plus,
            Box::new(curr_expr),
            Box::new(arr_size),
        )))));

        return attach_postfix_ops(tokens, equiv_deref_expr);
    } else {
        return curr_expr;
    }
}

fn parse_function_args(tokens: &mut TokenCursor) -> Vec<Expr> {
    let mut args = Vec::new();

    if tokens.peek() == Some(&Token::CloseParen) {
        return Vec::new();
    }
    loop {
        args.push(generate_expr_ast(
            tokens,
            BinOpPrecedenceLevel::lowest_level(),
        ));
        if tokens.peek() == Some(&Token::Comma) {
            tokens.next(); // consume the comma
        } else {
            break;
        }
    }

    args
}
