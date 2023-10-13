use crate::{
    parser::expr_parser::{BinOp, Expr},
    tac::get_new_label_number,
};

use super::{
    get_new_temp_name,
    prefix_postfix_inc_dec::{
        gen_postfix_dec_tac, gen_postfix_inc_tac, gen_prefix_dec_tac, gen_prefix_inc_tac,
    },
    resolve_variable_to_temp_name, CodeEnv, Identifier, TacInstr, TacVal,
};

pub fn generate_expr_tac(
    expr: &Expr,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    // returns a list of instructions to calculate an expression,
    // and the tacval (may be a var or an literal) containing the expression.

    // if target_temp_name is None, then this function will allocate a new temporary if required.
    // otherwise, it will store the result in target_temp_name.

    match expr {
        Expr::Var(var_name) => {
            if let Some(target_temp_name) = target_temp_name {
                return (
                    vec![TacInstr::Copy(
                        target_temp_name,
                        TacVal::Var(resolve_variable_to_temp_name(var_name, code_env)),
                    )],
                    TacVal::Var(target_temp_name),
                );
            }
            return (
                vec![],
                TacVal::Var(resolve_variable_to_temp_name(var_name, code_env)),
            );
        }
        Expr::Assign(var_name, expr) => {
            let temp_name_of_assignee = resolve_variable_to_temp_name(var_name, code_env);

            let (mut result, tac_val) =
                generate_expr_tac(expr, code_env, Some(temp_name_of_assignee));
            if let Some(ident) = target_temp_name {
                result.push(TacInstr::Copy(ident, tac_val));
                (result, TacVal::Var(ident))
            } else {
                (result, TacVal::Var(temp_name_of_assignee))
            }
        }
        Expr::Int(v) => {
            if let Some(ident) = target_temp_name {
                (
                    vec![TacInstr::Copy(ident, TacVal::Lit(*v))],
                    TacVal::Var(ident),
                )
            } else {
                (vec![], TacVal::Lit(*v))
            }
        }
        Expr::UnOp(op, inner_expr) => {
            let final_temp_name = if let Some(ident) = target_temp_name {
                ident
            } else {
                get_new_temp_name()
            };
            let (mut result, inner_val) = generate_expr_tac(inner_expr, code_env, None);
            result.push(TacInstr::UnOp(final_temp_name, inner_val, *op));
            (result, TacVal::Var(final_temp_name))
        }
        Expr::BinOp(op, expr1, expr2) => {
            generate_binop_tac(*op, expr1, expr2, code_env, target_temp_name)
        }
        Expr::Ternary(decision_expr, expr1, expr2) => {
            generate_ternary_tac(decision_expr, expr1, expr2, code_env, target_temp_name)
        }
        Expr::PostfixInc(var) => gen_postfix_inc_tac(var, code_env, target_temp_name),
        Expr::PostfixDec(var) => gen_postfix_dec_tac(var, code_env, target_temp_name),
        Expr::PrefixInc(var) => gen_prefix_inc_tac(var, code_env, target_temp_name),
        Expr::PrefixDec(var) => gen_prefix_dec_tac(var, code_env, target_temp_name),
    }
}

fn generate_binop_tac(
    op: BinOp,
    expr1: &Expr,
    expr2: &Expr,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    if op == BinOp::LogicalAnd || op == BinOp::LogicalOr {
        return generate_short_circuiting_tac(op, expr1, expr2, code_env, target_temp_name);
    }

    let final_temp_name: Identifier = if let Some(ident) = target_temp_name {
        ident
    } else {
        get_new_temp_name()
    };
    let (mut result, expr_1_val) = generate_expr_tac(expr1, code_env, None);
    let (result2, expr_2_val) = generate_expr_tac(expr2, code_env, None);

    result.extend(result2);
    result.push(TacInstr::Binop(final_temp_name, expr_1_val, expr_2_val, op));
    (result, TacVal::Var(final_temp_name))
}

fn generate_short_circuiting_tac(
    op: BinOp,
    expr1: &Expr,
    expr2: &Expr,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    let final_temp_name = if let Some(ident) = target_temp_name {
        ident
    } else {
        get_new_temp_name()
    };
    match op {
        BinOp::LogicalAnd => {
            let label_num = get_new_label_number();
            let label_and_false = format!("label_and_false_{}", label_num);
            let label_and_end = format!("label_and_end_{}", label_num);

            let (mut result, lhs_val) = generate_expr_tac(expr1, code_env, None);
            result.push(TacInstr::JmpZero(label_and_false.clone(), lhs_val));
            let (res_rhs, rhs_val) = generate_expr_tac(expr2, code_env, None);
            result.extend(res_rhs);
            result.push(TacInstr::Binop(
                final_temp_name,
                rhs_val,
                TacVal::Lit(0),
                BinOp::NotEquals,
            ));
            result.push(TacInstr::Jmp(label_and_end.clone()));
            result.push(TacInstr::Label(label_and_false));
            result.push(TacInstr::Copy(final_temp_name, TacVal::Lit(0)));
            result.push(TacInstr::Label(label_and_end));

            (result, TacVal::Var(final_temp_name))
        }
        BinOp::LogicalOr => {
            let label_num = get_new_label_number();
            let label_or_true = format!("label_or_true_{}", label_num);
            let label_or_end = format!("label_or_end_{}", label_num);

            let (mut result, lhs_val) = generate_expr_tac(expr1, code_env, None);
            result.push(TacInstr::JmpNotZero(label_or_true.clone(), lhs_val));
            let (res_rhs, rhs_val) = generate_expr_tac(expr2, code_env, None);
            result.extend(res_rhs);
            result.push(TacInstr::Binop(
                final_temp_name,
                rhs_val,
                TacVal::Lit(0),
                BinOp::NotEquals,
            ));
            result.push(TacInstr::Jmp(label_or_end.clone()));
            result.push(TacInstr::Label(label_or_true));
            result.push(TacInstr::Copy(final_temp_name, TacVal::Lit(1)));
            result.push(TacInstr::Label(label_or_end));

            (result, TacVal::Var(final_temp_name))
        }
        _ => unreachable!(),
    }
}

fn generate_ternary_tac(
    decision_expr: &Expr,
    expr1: &Expr,
    expr2: &Expr,
    code_env: &CodeEnv,
    target_temp_name: Option<Identifier>,
) -> (Vec<TacInstr>, TacVal) {
    let final_temp_name = if let Some(ident) = target_temp_name {
        ident
    } else {
        get_new_temp_name()
    };

    let label_num = get_new_label_number();
    let label_false = format!("label_ternary_false_{}", label_num);
    let label_end = format!("label_ternary_end_{}", label_num);

    let (mut result, decision_val) = generate_expr_tac(decision_expr, code_env, None);
    result.push(TacInstr::JmpZero(label_false.clone(), decision_val));

    let (res_expr1, _) = generate_expr_tac(expr1, code_env, Some(final_temp_name));
    result.extend(res_expr1);
    result.push(TacInstr::Jmp(label_end.clone()));

    result.push(TacInstr::Label(label_false));
    let (res_expr2, _) = generate_expr_tac(expr2, code_env, Some(final_temp_name));
    result.extend(res_expr2);
    result.push(TacInstr::Label(label_end));

    (result, TacVal::Var(final_temp_name))
}
