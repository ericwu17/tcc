use crate::parser::expr_parser::ExprEnum;

use super::expr_parser::{Expr, UnOp};

pub fn eval_unop(op: UnOp, expr: Expr) -> Expr {
    let val = match expr.content {
        ExprEnum::Int(v) => v,
        _ => unreachable!(),
    };

    let new_val;
    match op {
        UnOp::Negation => {
            new_val = -val;
        }
        UnOp::BitwiseComplement => new_val = !val,
        UnOp::Not => {
            new_val = if val == 0 { 1 } else { 0 };
        }
    }

    Expr {
        content: ExprEnum::Int(new_val),
        type_: expr.type_,
    }
}
