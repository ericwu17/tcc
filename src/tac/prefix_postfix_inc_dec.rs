use crate::parser::expr_parser::BinOp;

use super::{
    get_new_temp_name, resolve_variable_to_temp_name, CodeEnv, Identifier, TacInstr, TacVal,
};

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
        TacVal::Lit(1),
        BinOp::Plus,
    ));
    if let Some(ident) = target_temp_name {
        result.push(TacInstr::Copy(ident, TacVal::Var(temporary_ident)));
    }
    (result, TacVal::Var(temporary_ident))
}

pub fn gen_prefix_dec_tac(
    var_name: &String,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    let mut result = Vec::new();
    let temporary_ident = resolve_variable_to_temp_name(var_name, code_env);
    result.push(TacInstr::BinOp(
        temporary_ident,
        TacVal::Var(temporary_ident),
        TacVal::Lit(1),
        BinOp::Minus,
    ));
    if let Some(ident) = target_temp_name {
        result.push(TacInstr::Copy(ident, TacVal::Var(temporary_ident)));
    }
    (result, TacVal::Var(temporary_ident))
}

pub fn gen_postfix_inc_tac(
    var_name: &String,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    let mut result = Vec::new();
    let new_ident = if let Some(ident) = target_temp_name {
        ident
    } else {
        get_new_temp_name()
    };
    let temporary_ident = resolve_variable_to_temp_name(var_name, code_env);

    result.push(TacInstr::Copy(new_ident, TacVal::Var(temporary_ident)));

    result.push(TacInstr::BinOp(
        temporary_ident,
        TacVal::Var(temporary_ident),
        TacVal::Lit(1),
        BinOp::Plus,
    ));
    (result, TacVal::Var(new_ident))
}

pub fn gen_postfix_dec_tac(
    var_name: &String,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    let mut result = Vec::new();
    let new_ident = if let Some(ident) = target_temp_name {
        ident
    } else {
        get_new_temp_name()
    };
    let temporary_ident = resolve_variable_to_temp_name(var_name, code_env);

    result.push(TacInstr::Copy(new_ident, TacVal::Var(temporary_ident)));

    result.push(TacInstr::BinOp(
        temporary_ident,
        TacVal::Var(temporary_ident),
        TacVal::Lit(1),
        BinOp::Minus,
    ));
    (result, TacVal::Var(new_ident))
}
