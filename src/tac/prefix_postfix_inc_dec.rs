use std::path::Prefix;

use crate::{
    parser::expr_parser::{BinOp, Expr},
    types::VarSize,
};

use super::{
    expr::ValTarget, get_new_temp_name, resolve_variable_to_temp_name, CodeEnv, Identifier,
    TacInstr, TacVal,
};

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
    let should_return_old_val = match op {
        Operation::PrefixInc | Operation::PrefixDec => false,
        Operation::PostfixInc | Operation::PostfixDec => true,
    };
    let binary_op = match op {
        Operation::PrefixInc | Operation::PostfixInc => BinOp::Plus,
        Operation::PrefixDec | Operation::PostfixDec => BinOp::Minus,
    };

    match target {
        ValTarget::None => match expr {
            Expr::Var(var_name) => {
                let mut result = Vec::new();
                let ident_to_update = resolve_variable_to_temp_name(var_name, code_env);
                result.push(TacInstr::BinOp(
                    ident_to_update,
                    TacVal::Var(ident_to_update),
                    TacVal::Lit(1, ident_to_update.1),
                    binary_op,
                ));
                return (result, TacVal::Lit(0, VarSize::default()));
            }
            _ => todo!(),
        },
        ValTarget::Generate | ValTarget::Ident(_) => match expr {
            Expr::Var(var_name) => {
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
                result.push(TacInstr::BinOp(
                    ident_to_update,
                    TacVal::Var(ident_to_update),
                    TacVal::Lit(1, ident_to_update.1),
                    binary_op,
                ));

                if !should_return_old_val {
                    if let ValTarget::Ident(ident) = target {
                        result.push(TacInstr::Copy(ident, TacVal::Var(ident_to_update)));
                        ident_to_return = ident;
                    }
                }

                return (result, TacVal::Var(ident_to_return));
            }
            _ => todo!(),
        },
    }
}

pub fn gen_prefix_inc_tac(
    var_name: &String,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    let mut result = Vec::new();
    let temporary_ident = resolve_variable_to_temp_name(var_name, code_env);
    result.push(TacInstr::BinOp(
        temporary_ident,
        TacVal::Var(temporary_ident),
        TacVal::Lit(1, temporary_ident.1),
        BinOp::Plus,
    ));
    if let Some(ident) = target_temp_name {
        result.push(TacInstr::Copy(ident, TacVal::Var(temporary_ident)));
    }
    (result, TacVal::Var(temporary_ident))
}

// pub fn gen_prefix_dec_tac(
//     var_name: &String,
//     code_env: &CodeEnv,
//     target_temp_name: Option<Identifier>,
// ) -> (Vec<TacInstr>, TacVal) {
//     let mut result = Vec::new();
//     let temporary_ident = resolve_variable_to_temp_name(var_name, code_env);
//     result.push(TacInstr::BinOp(
//         temporary_ident,
//         TacVal::Var(temporary_ident),
//         TacVal::Lit(1, temporary_ident.1),
//         BinOp::Minus,
//     ));
//     if let Some(ident) = target_temp_name {
//         result.push(TacInstr::Copy(ident, TacVal::Var(temporary_ident)));
//     }
//     (result, TacVal::Var(temporary_ident))
// }

pub fn gen_postfix_inc_tac(
    var_name: &String,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    let mut result = Vec::new();
    let new_ident = if let Some(ident) = target_temp_name {
        ident
    } else {
        get_new_temp_name(resolve_variable_to_temp_name(var_name, code_env).1)
    };
    let temporary_ident = resolve_variable_to_temp_name(var_name, code_env);

    result.push(TacInstr::Copy(new_ident, TacVal::Var(temporary_ident)));

    result.push(TacInstr::BinOp(
        temporary_ident,
        TacVal::Var(temporary_ident),
        TacVal::Lit(1, new_ident.1),
        BinOp::Plus,
    ));
    (result, TacVal::Var(new_ident))
}

// pub fn gen_postfix_dec_tac(
//     var_name: &String,
//     code_env: &CodeEnv,
//     target_temp_name: Option<Identifier>,
// ) -> (Vec<TacInstr>, TacVal) {
//     let mut result = Vec::new();
//     let new_ident = if let Some(ident) = target_temp_name {
//         ident
//     } else {
//         get_new_temp_name(resolve_variable_to_temp_name(var_name, code_env).1)
//     };
//     let temporary_ident = resolve_variable_to_temp_name(var_name, code_env);

//     result.push(TacInstr::Copy(new_ident, TacVal::Var(temporary_ident)));

//     result.push(TacInstr::BinOp(
//         temporary_ident,
//         TacVal::Var(temporary_ident),
//         TacVal::Lit(1, new_ident.1),
//         BinOp::Minus,
//     ));
//     (result, TacVal::Var(new_ident))
// }
