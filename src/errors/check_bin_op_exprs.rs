use crate::{
    parser::expr_parser::{BinOp, Expr},
    types::{FundT, VarType},
};

use super::{
    check_types::{
        are_assignment_compatible_types, are_interchangable_types, get_type, is_l_value, CodeEnv,
    },
    display::err_display_no_source,
};

pub fn get_binop_type(
    op: BinOp,
    expr1: &mut Expr,
    expr2: &mut Expr,
    code_env: &CodeEnv,
) -> Option<VarType> {
    let t1 = get_type(expr1, code_env);
    let t2 = get_type(expr2, code_env);

    match op {
        BinOp::Multiply | BinOp::Divide | BinOp::Modulus | BinOp::LogicalAnd | BinOp::LogicalOr => {
            // These operations require 2 fundamental types
            let mut error = false;
            if let Some(VarType::Ptr(_)) = t1 {
                error = true;
            }
            if let Some(VarType::Arr(_, _)) = t1 {
                error = true;
            }
            if let Some(VarType::Ptr(_)) = t2 {
                error = true;
            }
            if let Some(VarType::Arr(_, _)) = t2 {
                error = true;
            }
            if error {
                err_display_no_source("expected integer in *, /, %, &&, or ||")
            }
            t1
        }

        BinOp::Plus | BinOp::Minus => match (&t1, &t2) {
            (None | Some(VarType::Fund(_)), None | Some(VarType::Fund(_))) => t1,
            (None | Some(VarType::Fund(_)), Some(t)) | (Some(t), None | Some(VarType::Fund(_))) => {
                match t {
                    VarType::Fund(_) => Some(t.clone()),
                    VarType::Ptr(_) => Some(t.clone()),
                    VarType::Arr(array_inner_type, _) => {
                        Some(VarType::Ptr(array_inner_type.clone()))
                    }
                }
            }
            (Some(VarType::Ptr(t1)), Some(VarType::Ptr(t2)))
                if op == BinOp::Minus && t1.num_bytes() == t2.num_bytes() =>
            {
                // subtracting two pointers gives an integer
                Some(VarType::Fund(FundT::Long))
            }
            (Some(t1), Some(t2)) => err_display_no_source(format!(
                "trying to add or subtract incompatible types {} and {}",
                t1, t2
            )),
        },
        BinOp::GreaterThan
        | BinOp::GreaterThanEq
        | BinOp::LessThan
        | BinOp::LessThanEq
        | BinOp::Equals
        | BinOp::NotEquals => {
            if !are_interchangable_types(&t1, &t2) {
                err_display_no_source(format!(
                    "trying to compare incompatible types {:?} and {:?}",
                    t1, t2
                ));
            }
            t1
        }

        BinOp::Assign => {
            if !is_l_value(expr1) {
                err_display_no_source("cannot assign to a non l value");
            }
            if !are_assignment_compatible_types(&t1, &t2) {
                err_display_no_source("wrong types in assignment.")
            }
            t1
        }
    }
}
