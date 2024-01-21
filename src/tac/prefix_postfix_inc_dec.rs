use crate::{
    parser::expr_parser::{BinOp, Expr, ExprEnum},
    tac::expr::{generate_expr_tac, get_pointee_size},
    types::{VarSize, VarType},
};

use super::{
    expr::{get_pointee_type, ValTarget},
    Identifier,
};
use super::{get_new_temp_name, resolve_variable_to_temp_name, CodeEnv, TacInstr, TacVal};

pub enum Operation {
    PrefixInc,
    PrefixDec,
    PostfixInc,
    PostfixDec,
}

pub fn gen_prefix_postfix_inc_dec(
    expr: &Expr,
    op: Operation,
    code_env: &CodeEnv,
    target: ValTarget,
) -> (Vec<TacInstr>, TacVal) {
    let mut should_return_old_val = match op {
        Operation::PrefixInc | Operation::PrefixDec => false,
        Operation::PostfixInc | Operation::PostfixDec => true,
    };
    let binary_op = match op {
        Operation::PrefixInc | Operation::PostfixInc => BinOp::Plus,
        Operation::PrefixDec | Operation::PostfixDec => BinOp::Minus,
    };
    if target == ValTarget::None {
        // if we don't need to return anything anyway, it's slightly more efficient
        // to just return the new value (no need for an extra temporary to store old value)
        should_return_old_val = false;
    }

    match &expr.content {
        ExprEnum::Var(var_name) => {
            let mut result = Vec::new();
            let ident_to_update = resolve_variable_to_temp_name(var_name, code_env);
            let mut ident_to_return;
            if should_return_old_val {
                if let ValTarget::Ident(ident) = target {
                    ident_to_return = ident;
                } else {
                    ident_to_return =
                        get_new_temp_name(resolve_variable_to_temp_name(var_name, code_env).1);
                };
                result.push(TacInstr::Copy(
                    ident_to_return,
                    TacVal::Var(ident_to_update),
                ));
            } else {
                ident_to_return = ident_to_update;
            }
            result.push(generate_update_code(
                ident_to_update,
                expr.type_.clone().unwrap(),
                binary_op,
            ));

            if !should_return_old_val {
                if let ValTarget::Ident(ident) = target {
                    result.push(TacInstr::Copy(ident, TacVal::Var(ident_to_update)));
                    ident_to_return = ident;
                }
            }

            (result, TacVal::Var(ident_to_return))
        }
        ExprEnum::Deref(ptr_expr) => {
            let (mut result, tac_val_1) =
                generate_expr_tac(ptr_expr, code_env, ValTarget::Generate);
            let pointee_size = get_pointee_size(&ptr_expr.type_.clone().unwrap()).unwrap();
            let pointee_type = get_pointee_type(&ptr_expr.type_.clone().unwrap());
            let temp_ident = get_new_temp_name(pointee_size);
            let mut ident_to_return;

            if let TacVal::Var(ident) = tac_val_1 {
                result.push(TacInstr::Deref(temp_ident, ident));

                if should_return_old_val {
                    if let ValTarget::Ident(ident) = target {
                        ident_to_return = ident;
                    } else {
                        ident_to_return = get_new_temp_name(pointee_size);
                    };
                    result.push(TacInstr::Copy(ident_to_return, TacVal::Var(temp_ident)));
                } else {
                    ident_to_return = temp_ident;
                }

                result.push(generate_update_code(temp_ident, pointee_type, binary_op));
                result.push(TacInstr::DerefStore(ident, TacVal::Var(temp_ident)));

                if !should_return_old_val {
                    if let ValTarget::Ident(ident) = target {
                        result.push(TacInstr::Copy(ident, TacVal::Var(temp_ident)));
                        ident_to_return = ident;
                    }
                }
            } else {
                unreachable!();
            }

            (result, TacVal::Var(ident_to_return))
        }
        _ => unreachable!(),
    }
}

fn generate_update_code(ident_to_update: Identifier, type_: VarType, binary_op: BinOp) -> TacInstr {
    let change_amt = match type_ {
        VarType::Fund(_) => 1,
        VarType::Ptr(inner) | VarType::Arr(inner, _) => inner.num_bytes(),
    };
    TacInstr::BinOp(
        ident_to_update,
        TacVal::Var(ident_to_update),
        TacVal::Lit(change_amt as i64, VarSize::Quad),
        binary_op,
    )
}
