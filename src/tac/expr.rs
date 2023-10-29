use crate::{
    parser::expr_parser::{BinOp, Expr},
    tac::get_new_label_number,
};

use super::{
    get_new_temp_name,
    prefix_postfix_inc_dec::{gen_prefix_postfix_inc_dec, Operation},
    resolve_variable_to_temp_name, CodeEnv, Identifier, TacInstr, TacVal, VarSize,
};

#[derive(PartialEq, Clone, Copy)]
pub enum ValTarget {
    None,              // evaluate the expression only for side effects
    Generate,          // generate one if required, but return TacVal (which might be a literal)
    Ident(Identifier), // write the value of the expression into this identifier
}

pub fn generate_expr_tac(
    expr: &Expr,
    code_env: &CodeEnv,
    target: ValTarget,
    suggested_size: Option<VarSize>,
) -> (Vec<TacInstr>, TacVal) {
    // returns a list of instructions to calculate an expression,
    // and the tacval (may be a var or an literal) containing the expression.

    // if the caller supplies ValTarget::None for target, then the caller
    // should also ignore the TacVal returned, since the caller is evaluating the expr for side
    // effects only anyway

    match expr {
        Expr::Var(var_name) => {
            if let ValTarget::Ident(target_temp_name) = target {
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
        Expr::Int(v) => match target {
            ValTarget::Generate | ValTarget::None => {
                (vec![], TacVal::Lit(*v, suggested_size.unwrap_or_default()))
            }
            ValTarget::Ident(ident) => (
                vec![TacInstr::Copy(ident, TacVal::Lit(*v, ident.1))],
                TacVal::Var(ident),
            ),
        },
        Expr::UnOp(op, inner_expr) => match target {
            ValTarget::Generate | ValTarget::Ident(_) => {
                let final_temp_name = if let ValTarget::Ident(ident) = target {
                    ident
                } else {
                    get_new_temp_name(
                        get_expr_size(inner_expr, code_env)
                            .unwrap_or(suggested_size.unwrap_or_default()),
                    )
                };
                let (mut result, inner_val) =
                    generate_expr_tac(inner_expr, code_env, ValTarget::Generate, suggested_size);
                result.push(TacInstr::UnOp(final_temp_name, inner_val, *op));
                (result, TacVal::Var(final_temp_name))
            }
            ValTarget::None => {
                generate_expr_tac(inner_expr, code_env, ValTarget::None, suggested_size)
            }
        },
        Expr::BinOp(op, expr1, expr2) => {
            generate_binop_tac(*op, expr1, expr2, code_env, target, suggested_size)
        }
        Expr::Ternary(decision_expr, expr1, expr2) => generate_ternary_tac(
            decision_expr,
            expr1,
            expr2,
            code_env,
            target,
            suggested_size,
        ),

        Expr::PostfixInc(var) => {
            gen_prefix_postfix_inc_dec(var, Operation::PostfixInc, code_env, target)
        }
        Expr::PostfixDec(var) => {
            gen_prefix_postfix_inc_dec(var, Operation::PostfixDec, code_env, target)
        }
        Expr::PrefixInc(var) => {
            gen_prefix_postfix_inc_dec(var, Operation::PrefixInc, code_env, target)
        }
        Expr::PrefixDec(var) => {
            gen_prefix_postfix_inc_dec(var, Operation::PrefixDec, code_env, target)
        }

        Expr::FunctionCall(func_ident, args) => {
            gen_function_call_tac(func_ident, args, code_env, target)
        }
        Expr::Deref(_) => todo!(),
        Expr::Ref(_) => todo!(),
        Expr::Sizeof(_) => unreachable!(), // sizeof should have been replaced by int literal by check_types
    }
}

fn generate_binop_tac(
    op: BinOp,
    expr1: &Expr,
    expr2: &Expr,
    code_env: &CodeEnv,
    target: ValTarget,
    suggested_size: Option<VarSize>,
) -> (Vec<TacInstr>, TacVal) {
    if op == BinOp::LogicalAnd || op == BinOp::LogicalOr {
        return generate_short_circuiting_tac(op, expr1, expr2, code_env, target, suggested_size);
    }

    if op == BinOp::Assign {
        return generate_assignment_tac(expr1, expr2, code_env, target);
    }
    match target {
        ValTarget::Generate | ValTarget::Ident(_) => {
            let final_temp_name = if let ValTarget::Ident(ident) = target {
                ident
            } else {
                get_new_temp_name(
                    get_bigger_size(
                        get_expr_size(expr1, code_env),
                        get_expr_size(expr2, code_env),
                    )
                    .unwrap_or(suggested_size.unwrap_or_default()),
                )
            };
            let (mut result, expr_1_val) =
                generate_expr_tac(expr1, code_env, ValTarget::Generate, suggested_size);
            let (result2, expr_2_val) =
                generate_expr_tac(expr2, code_env, ValTarget::Generate, suggested_size);

            result.extend(result2);
            result.push(TacInstr::BinOp(final_temp_name, expr_1_val, expr_2_val, op));
            (result, TacVal::Var(final_temp_name))
        }
        ValTarget::None => {
            let (mut result, _) =
                generate_expr_tac(expr1, code_env, ValTarget::None, suggested_size);
            let (result2, val) =
                generate_expr_tac(expr2, code_env, ValTarget::None, suggested_size);
            result.extend(result2);
            (result, val)
        }
    }
}

fn generate_short_circuiting_tac(
    op: BinOp,
    expr1: &Expr,
    expr2: &Expr,
    code_env: &CodeEnv,
    target: ValTarget,
    suggested_size: Option<VarSize>,
) -> (Vec<TacInstr>, TacVal) {
    let final_temp_name = if let ValTarget::Ident(ident) = target {
        ident
    } else {
        get_new_temp_name(
            get_bigger_size(
                get_expr_size(expr1, code_env),
                get_expr_size(expr2, code_env),
            )
            .unwrap_or(suggested_size.unwrap_or_default()),
        )
    };

    match op {
        BinOp::LogicalAnd => {
            let label_num = get_new_label_number();
            let label_and_false = format!("label_and_false_{}", label_num);
            let label_and_end = format!("label_and_end_{}", label_num);

            let (mut result, lhs_val) =
                generate_expr_tac(expr1, code_env, ValTarget::Generate, None);
            result.push(TacInstr::JmpZero(label_and_false.clone(), lhs_val));
            let (res_rhs, rhs_val) = generate_expr_tac(expr2, code_env, ValTarget::Generate, None);
            result.extend(res_rhs);
            result.push(TacInstr::BinOp(
                final_temp_name,
                rhs_val,
                TacVal::Lit(0, final_temp_name.1),
                BinOp::NotEquals,
            ));
            result.push(TacInstr::Jmp(label_and_end.clone()));
            result.push(TacInstr::Label(label_and_false));
            result.push(TacInstr::Copy(
                final_temp_name,
                TacVal::Lit(0, final_temp_name.1),
            ));
            result.push(TacInstr::Label(label_and_end));

            (result, TacVal::Var(final_temp_name))
        }
        BinOp::LogicalOr => {
            let label_num = get_new_label_number();
            let label_or_true = format!("label_or_true_{}", label_num);
            let label_or_end = format!("label_or_end_{}", label_num);

            let (mut result, lhs_val) =
                generate_expr_tac(expr1, code_env, ValTarget::Generate, None);
            result.push(TacInstr::JmpNotZero(label_or_true.clone(), lhs_val));
            let (res_rhs, rhs_val) = generate_expr_tac(expr2, code_env, ValTarget::Generate, None);
            result.extend(res_rhs);
            result.push(TacInstr::BinOp(
                final_temp_name,
                rhs_val,
                TacVal::Lit(0, final_temp_name.1),
                BinOp::NotEquals,
            ));
            result.push(TacInstr::Jmp(label_or_end.clone()));
            result.push(TacInstr::Label(label_or_true));
            result.push(TacInstr::Copy(
                final_temp_name,
                TacVal::Lit(1, final_temp_name.1),
            ));
            result.push(TacInstr::Label(label_or_end));

            (result, TacVal::Var(final_temp_name))
        }
        _ => unreachable!(),
    }
}

fn generate_assignment_tac(
    lhs: &Expr,
    rhs: &Expr,
    code_env: &CodeEnv,
    target: ValTarget,
) -> (Vec<TacInstr>, TacVal) {
    match lhs {
        Expr::Var(var_name) => {
            let temp_name_of_assignee = resolve_variable_to_temp_name(var_name, code_env);

            let (mut result, tac_val) = generate_expr_tac(
                rhs,
                code_env,
                ValTarget::Ident(temp_name_of_assignee),
                Some(temp_name_of_assignee.1),
            );
            if let ValTarget::Ident(ident) = target {
                result.push(TacInstr::Copy(ident, tac_val));
                (result, TacVal::Var(ident))
            } else {
                (result, TacVal::Var(temp_name_of_assignee))
            }
        }
        _ => todo!(),
    }
}

fn generate_ternary_tac(
    decision_expr: &Expr,
    expr1: &Expr,
    expr2: &Expr,
    code_env: &CodeEnv,
    target: ValTarget,
    suggested_size: Option<VarSize>,
) -> (Vec<TacInstr>, TacVal) {
    let final_temp_name = if let ValTarget::Ident(ident) = target {
        ident
    } else {
        get_new_temp_name(
            get_bigger_size(
                get_expr_size(expr1, code_env),
                get_expr_size(expr2, code_env),
            )
            .unwrap_or(suggested_size.unwrap_or_default()),
        )
    };

    let label_num = get_new_label_number();
    let label_false = format!("label_ternary_false_{}", label_num);
    let label_end = format!("label_ternary_end_{}", label_num);

    let (mut result, decision_val) =
        generate_expr_tac(decision_expr, code_env, ValTarget::Generate, None);
    result.push(TacInstr::JmpZero(label_false.clone(), decision_val));

    let (res_expr1, _) = generate_expr_tac(
        expr1,
        code_env,
        ValTarget::Ident(final_temp_name),
        Some(final_temp_name.1),
    );
    result.extend(res_expr1);
    result.push(TacInstr::Jmp(label_end.clone()));

    result.push(TacInstr::Label(label_false));
    let (res_expr2, _) = generate_expr_tac(
        expr2,
        code_env,
        ValTarget::Ident(final_temp_name),
        Some(final_temp_name.1),
    );
    result.extend(res_expr2);
    result.push(TacInstr::Label(label_end));

    (result, TacVal::Var(final_temp_name))
}

pub fn gen_function_call_tac(
    func_ident: &String,
    args: &Vec<Expr>,
    code_env: &CodeEnv,
    target: ValTarget,
) -> (Vec<TacInstr>, TacVal) {
    let final_temp_name = if let ValTarget::Ident(ident) = target {
        ident
    } else {
        get_new_temp_name(VarSize::default())
    };

    let mut result = Vec::new();
    let mut arg_vals = Vec::new();

    for arg_expr in args {
        let (instrs, arg_val) = generate_expr_tac(arg_expr, code_env, ValTarget::Generate, None);
        result.extend(instrs);
        arg_vals.push(arg_val);
    }

    result.push(TacInstr::Call(
        func_ident.clone(),
        arg_vals,
        Some(final_temp_name),
    ));

    (result, TacVal::Var(final_temp_name))
}

pub fn get_bigger_size(s1: Option<VarSize>, s2: Option<VarSize>) -> Option<VarSize> {
    if s1 == Some(VarSize::Quad) || s2 == Some(VarSize::Quad) {
        return Some(VarSize::Quad);
    }
    if s1 == Some(VarSize::Dword) || s2 == Some(VarSize::Dword) {
        return Some(VarSize::Dword);
    }
    if s1 == Some(VarSize::Word) || s2 == Some(VarSize::Word) {
        return Some(VarSize::Word);
    }

    if s1 == Some(VarSize::Byte) || s2 == Some(VarSize::Byte) {
        return Some(VarSize::Byte);
    }

    return None;
}

pub fn get_expr_size(expr: &Expr, code_env: &CodeEnv) -> Option<VarSize> {
    let res = match expr {
        Expr::Int(_) => None,
        Expr::Var(name) => Some(resolve_variable_to_temp_name(name, code_env).1),
        // Expr::Assign(name, _) => Some(resolve_variable_to_temp_name(name, code_env).1),
        Expr::UnOp(_, inner_expr) => get_expr_size(inner_expr, code_env),
        Expr::BinOp(_, inner_expr_1, inner_expr_2) => get_bigger_size(
            get_expr_size(inner_expr_1, code_env),
            get_expr_size(inner_expr_2, code_env),
        ),
        Expr::Ternary(_, inner_expr_1, inner_expr_2) => get_bigger_size(
            get_expr_size(inner_expr_1, code_env),
            get_expr_size(inner_expr_2, code_env),
        ),
        Expr::FunctionCall(_, _) => Some(VarSize::default()),
        Expr::Deref(_) => todo!(),
        Expr::Ref(_) => todo!(),
        Expr::PostfixDec(inner_expr)
        | Expr::PostfixInc(inner_expr)
        | Expr::PrefixDec(inner_expr)
        | Expr::PrefixInc(inner_expr) => get_expr_size(inner_expr, code_env),
        Expr::Sizeof(_) => Some(VarSize::default()),
    };

    if res == Some(VarSize::Quad) {
        return res;
    }
    return Some(VarSize::Dword);
}
