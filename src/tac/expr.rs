use crate::{
    parser::expr_parser::{BinOp, Expr, ExprEnum},
    tac::get_new_label_number,
    types::VarType,
};

use super::{
    get_new_temp_name,
    prefix_postfix_inc_dec::{gen_prefix_postfix_inc_dec, Operation},
    resolve_variable_to_temp_name, CodeEnv, Identifier, TacInstr, TacVal, VarSize,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ValTarget {
    None,              // evaluate the expression only for side effects
    Generate,          // generate one if required, but return TacVal (which might be a literal)
    Ident(Identifier), // write the value of the expression into this identifier
}

pub fn generate_expr_tac(
    expr: &Expr,
    code_env: &CodeEnv,
    target: ValTarget,
) -> (Vec<TacInstr>, TacVal) {
    // returns a list of instructions to calculate an expression,
    // and the tacval (may be a var or an literal) containing the expression.

    // if the caller supplies ValTarget::None for target, then the caller
    // should also ignore the TacVal returned, since the caller is evaluating the expr for side
    // effects only anyway

    match &expr.content {
        ExprEnum::Var(var_name) => {
            if let ValTarget::Ident(target_temp_name) = target {
                return (
                    vec![TacInstr::Copy(
                        target_temp_name,
                        TacVal::Var(resolve_variable_to_temp_name(var_name, code_env)),
                    )],
                    TacVal::Var(target_temp_name),
                );
            }
            (
                vec![],
                TacVal::Var(resolve_variable_to_temp_name(var_name, code_env)),
            )
        }
        ExprEnum::Int(v) => match target {
            ValTarget::Generate | ValTarget::None => (vec![], TacVal::Lit(*v, VarSize::Quad)),
            ValTarget::Ident(ident) => (
                vec![TacInstr::Copy(ident, TacVal::Lit(*v, ident.1))],
                TacVal::Var(ident),
            ),
        },
        ExprEnum::UnOp(op, inner_expr) => match target {
            ValTarget::Generate | ValTarget::Ident(_) => {
                let final_temp_name = if let ValTarget::Ident(ident) = target {
                    ident
                } else {
                    get_new_temp_name(get_expr_size(inner_expr).unwrap_or_default())
                };
                let (mut result, inner_val) =
                    generate_expr_tac(inner_expr, code_env, ValTarget::Generate);
                result.push(TacInstr::UnOp(final_temp_name, inner_val, *op));
                (result, TacVal::Var(final_temp_name))
            }
            ValTarget::None => generate_expr_tac(inner_expr, code_env, ValTarget::None),
        },
        ExprEnum::BinOp(op, expr1, expr2) => {
            generate_binop_tac(*op, expr1, expr2, code_env, target)
        }
        ExprEnum::Ternary(decision_expr, expr1, expr2) => {
            generate_ternary_tac(decision_expr, expr1, expr2, code_env, target)
        }

        ExprEnum::PostfixInc(var) => {
            gen_prefix_postfix_inc_dec(var, Operation::PostfixInc, code_env, target)
        }
        ExprEnum::PostfixDec(var) => {
            gen_prefix_postfix_inc_dec(var, Operation::PostfixDec, code_env, target)
        }
        ExprEnum::PrefixInc(var) => {
            gen_prefix_postfix_inc_dec(var, Operation::PrefixInc, code_env, target)
        }
        ExprEnum::PrefixDec(var) => {
            gen_prefix_postfix_inc_dec(var, Operation::PrefixDec, code_env, target)
        }

        ExprEnum::FunctionCall(func_ident, args) => {
            gen_function_call_tac(func_ident, args, code_env, target)
        }
        ExprEnum::Deref(inner_expr) => {
            let (mut res, res_ident) = generate_expr_tac(inner_expr, code_env, ValTarget::Generate);
            let inner_expr_type = &inner_expr.type_.clone().unwrap();
            match target {
                ValTarget::None => (res, res_ident),
                ValTarget::Generate | ValTarget::Ident(_) => {
                    let final_temp_name = if let ValTarget::Ident(ident) = target {
                        ident
                    } else {
                        let pointee_size = get_pointee_size(inner_expr_type);
                        get_new_temp_name(pointee_size.unwrap_or_default())
                    };
                    if let TacVal::Var(ident) = res_ident {
                        if let VarType::Arr(_, _) = get_pointee_type(inner_expr_type) {
                            // if we have a pointer to an array, then dereferencing should  give a pointer _into_ the array (pointing at first element).
                            res.push(TacInstr::Copy(final_temp_name, TacVal::Var(ident)));
                        } else {
                            res.push(TacInstr::Deref(final_temp_name, ident));
                        }
                        (res, TacVal::Var(final_temp_name))
                    } else {
                        unreachable!()
                    }
                }
            }
        }
        ExprEnum::Ref(inner_exp) => match &inner_exp.content {
            ExprEnum::Deref(inner) => generate_expr_tac(inner, code_env, target),
            ExprEnum::Var(var_name) => {
                let final_temp_name = if let ValTarget::Ident(ident) = target {
                    ident
                } else {
                    get_new_temp_name(VarSize::Quad)
                };
                assert!(final_temp_name.1 == VarSize::Quad);
                let result = vec![TacInstr::Ref(
                    final_temp_name,
                    resolve_variable_to_temp_name(var_name, code_env),
                )];
                (result, TacVal::Var(final_temp_name))
            }
            _ => unreachable!(),
        },
        ExprEnum::StaticStrPtr(val) => match target {
            ValTarget::None => (vec![], TacVal::Lit(0, VarSize::Quad)),
            ValTarget::Generate | ValTarget::Ident(_) => {
                let final_temp_name = if let ValTarget::Ident(ident) = target {
                    ident
                } else {
                    get_new_temp_name(VarSize::Quad)
                };

                (
                    vec![TacInstr::StaticStrPtr(final_temp_name, val.clone())],
                    TacVal::Var(final_temp_name),
                )
            }
        },
        ExprEnum::Sizeof(_) => unreachable!(), // sizeof should have been replaced by int literal by check_types
        ExprEnum::ArrInitExpr(_) => unreachable!(), // ArrInitExpr should only appear in array initializations
    }
}

fn generate_binop_tac(
    op: BinOp,
    expr1: &Expr,
    expr2: &Expr,
    code_env: &CodeEnv,
    target: ValTarget,
) -> (Vec<TacInstr>, TacVal) {
    if op == BinOp::LogicalAnd || op == BinOp::LogicalOr {
        return generate_short_circuiting_tac(op, expr1, expr2, code_env, target);
    }

    if op == BinOp::Assign {
        return generate_assignment_tac(expr1, expr2, code_env, target);
    }
    match target {
        ValTarget::Generate | ValTarget::Ident(_) => {
            let final_temp_name = if let ValTarget::Ident(ident) = target {
                ident
            } else {
                // integer promotion rules
                get_new_temp_name(VarSize::Quad)
            };

            let (mut result, expr_1_val) = generate_expr_tac(expr1, code_env, ValTarget::Generate);
            let (result2, expr_2_val) = generate_expr_tac(expr2, code_env, ValTarget::Generate);

            result.extend(result2);

            if op == BinOp::Plus {
                let t1 = expr1.type_.clone().unwrap_or_default();
                let t2 = expr2.type_.clone().unwrap_or_default();
                result.extend(gen_addition_tac(
                    t1,
                    t2,
                    expr_1_val,
                    expr_2_val,
                    final_temp_name,
                ));
            } else if op == BinOp::Minus {
                let t1 = expr1.type_.clone().unwrap_or_default();
                let t2 = expr2.type_.clone().unwrap_or_default();
                result.extend(gen_subtraction_tac(
                    t1,
                    t2,
                    expr_1_val,
                    expr_2_val,
                    final_temp_name,
                ));
            } else {
                result.push(TacInstr::BinOp(final_temp_name, expr_1_val, expr_2_val, op));
            }
            (result, TacVal::Var(final_temp_name))
        }
        ValTarget::None => {
            let (mut result, _) = generate_expr_tac(expr1, code_env, ValTarget::None);
            let (result2, val) = generate_expr_tac(expr2, code_env, ValTarget::None);
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
) -> (Vec<TacInstr>, TacVal) {
    let final_temp_name = if let ValTarget::Ident(ident) = target {
        ident
    } else {
        get_new_temp_name(
            get_bigger_size(get_expr_size(expr1), get_expr_size(expr2)).unwrap_or_default(),
        )
    };

    match op {
        BinOp::LogicalAnd => {
            let label_num = get_new_label_number();
            let label_and_false = format!("label_and_false_{}", label_num);
            let label_and_end = format!("label_and_end_{}", label_num);

            let (mut result, lhs_val) = generate_expr_tac(expr1, code_env, ValTarget::Generate);
            result.push(TacInstr::JmpZero(label_and_false.clone(), lhs_val));
            let (res_rhs, rhs_val) = generate_expr_tac(expr2, code_env, ValTarget::Generate);
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

            let (mut result, lhs_val) = generate_expr_tac(expr1, code_env, ValTarget::Generate);
            result.push(TacInstr::JmpNotZero(label_or_true.clone(), lhs_val));
            let (res_rhs, rhs_val) = generate_expr_tac(expr2, code_env, ValTarget::Generate);
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
    match &lhs.content {
        ExprEnum::Var(var_name) => {
            let temp_name_of_assignee = resolve_variable_to_temp_name(var_name, code_env);

            let (mut result, tac_val) =
                generate_expr_tac(rhs, code_env, ValTarget::Ident(temp_name_of_assignee));
            if let ValTarget::Ident(ident) = target {
                result.push(TacInstr::Copy(ident, tac_val));
                (result, TacVal::Var(ident))
            } else {
                (result, TacVal::Var(temp_name_of_assignee))
            }
        }
        ExprEnum::Deref(inner) => {
            let (mut result, tac_val_1) = generate_expr_tac(inner, code_env, ValTarget::Generate);
            let (result2, tac_val_2) = generate_expr_tac(rhs, code_env, ValTarget::Generate);
            result.extend(result2);
            if let TacVal::Var(ident) = tac_val_1 {
                result.push(TacInstr::DerefStore(ident, tac_val_2.clone()));
            } else {
                unreachable!();
            }

            (result, tac_val_2)
        }
        _ => unreachable!(), // already checked that lhs must be a l_value
    }
}

fn generate_ternary_tac(
    decision_expr: &Expr,
    expr1: &Expr,
    expr2: &Expr,
    code_env: &CodeEnv,
    target: ValTarget,
) -> (Vec<TacInstr>, TacVal) {
    let final_temp_name = if let ValTarget::Ident(ident) = target {
        ident
    } else {
        get_new_temp_name(
            get_bigger_size(get_expr_size(expr1), get_expr_size(expr2)).unwrap_or_default(),
        )
    };

    let label_num = get_new_label_number();
    let label_false = format!("label_ternary_false_{}", label_num);
    let label_end = format!("label_ternary_end_{}", label_num);

    let (mut result, decision_val) =
        generate_expr_tac(decision_expr, code_env, ValTarget::Generate);
    result.push(TacInstr::JmpZero(label_false.clone(), decision_val));

    let (res_expr1, _) = generate_expr_tac(expr1, code_env, ValTarget::Ident(final_temp_name));
    result.extend(res_expr1);
    result.push(TacInstr::Jmp(label_end.clone()));

    result.push(TacInstr::Label(label_false));
    let (res_expr2, _) = generate_expr_tac(expr2, code_env, ValTarget::Ident(final_temp_name));
    result.extend(res_expr2);
    result.push(TacInstr::Label(label_end));

    (result, TacVal::Var(final_temp_name))
}

pub fn gen_function_call_tac(
    func_ident: &str,
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
        let (instrs, arg_val) = generate_expr_tac(arg_expr, code_env, ValTarget::Generate);
        result.extend(instrs);
        arg_vals.push(arg_val);
    }

    result.push(TacInstr::Call(
        func_ident.to_string(),
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

    None
}

pub fn get_expr_size(expr: &Expr) -> Option<VarSize> {
    let t = expr.type_.clone().unwrap_or_default();
    get_type_size(&t)
}

pub fn get_type_size(t: &VarType) -> Option<VarSize> {
    if let VarType::Arr(_, _) = t {
        // arrays are pointers, and therefore occupy a quad
        return Some(VarSize::Quad);
    }
    match t.num_bytes() {
        1 => Some(VarSize::Byte),
        2 => Some(VarSize::Word),
        4 => Some(VarSize::Dword),
        8 => Some(VarSize::Quad),
        _ => None,
    }
}

fn gen_addition_tac(
    t1: VarType,
    t2: VarType,
    val1: TacVal,
    val2: TacVal,
    final_temp_name: Identifier,
) -> Vec<TacInstr> {
    let mut result = Vec::new();
    match (&t1, &t2) {
        (VarType::Fund(_), VarType::Fund(_)) => {
            result.push(TacInstr::BinOp(final_temp_name, val1, val2, BinOp::Plus));
        }
        (VarType::Fund(_), VarType::Ptr(inner_type))
        | (VarType::Fund(_), VarType::Arr(inner_type, _)) => {
            let ptr_size = inner_type.num_bytes();
            let offset_number = get_new_temp_name(VarSize::Quad);
            result.push(TacInstr::BinOp(
                offset_number,
                val1,
                TacVal::Lit(ptr_size as i64, VarSize::Quad),
                BinOp::Multiply,
            ));
            result.push(TacInstr::BinOp(
                final_temp_name,
                TacVal::Var(offset_number),
                val2,
                BinOp::Plus,
            ));
        }
        (VarType::Ptr(inner_type), VarType::Fund(_))
        | (VarType::Arr(inner_type, _), VarType::Fund(_)) => {
            let ptr_size = inner_type.num_bytes();
            let offset_number = get_new_temp_name(VarSize::Quad);
            result.push(TacInstr::BinOp(
                offset_number,
                val2,
                TacVal::Lit(ptr_size as i64, VarSize::Quad),
                BinOp::Multiply,
            ));
            result.push(TacInstr::BinOp(
                final_temp_name,
                val1,
                TacVal::Var(offset_number),
                BinOp::Plus,
            ));
        }

        (VarType::Ptr(_), VarType::Ptr(_))
        | (VarType::Ptr(_), VarType::Arr(_, _))
        | (VarType::Arr(_, _), VarType::Ptr(_))
        | (VarType::Arr(_, _), VarType::Arr(_, _)) => unreachable!(),
    }
    result
}

pub fn gen_subtraction_tac(
    t1: VarType,
    t2: VarType,
    val1: TacVal,
    val2: TacVal,
    final_temp_name: Identifier,
) -> Vec<TacInstr> {
    let mut result = Vec::new();
    match (&t1, &t2) {
        (VarType::Fund(_), VarType::Fund(_)) => {
            result.push(TacInstr::BinOp(final_temp_name, val1, val2, BinOp::Minus));
        }
        (VarType::Ptr(inner_type_1), VarType::Ptr(inner_type_2)) => {
            assert_eq!(inner_type_1.num_bytes(), inner_type_2.num_bytes());
            let ptr_size = inner_type_1.num_bytes();
            result.push(TacInstr::BinOp(final_temp_name, val1, val2, BinOp::Minus));
            result.push(TacInstr::BinOp(
                final_temp_name,
                TacVal::Var(final_temp_name),
                TacVal::Lit(ptr_size as i64, VarSize::Quad),
                BinOp::Divide,
            ));
        }
        (VarType::Ptr(inner_type), VarType::Fund(_)) => {
            let ptr_size = inner_type.num_bytes();
            let offset_number = get_new_temp_name(VarSize::Quad);
            result.push(TacInstr::BinOp(
                offset_number,
                val2,
                TacVal::Lit(ptr_size as i64, VarSize::Quad),
                BinOp::Multiply,
            ));
            result.push(TacInstr::BinOp(
                final_temp_name,
                val1,
                TacVal::Var(offset_number),
                BinOp::Minus,
            ));
        }

        _ => unreachable!(),
    }
    result
}

pub fn get_pointee_size(t: &VarType) -> Option<VarSize> {
    match t {
        VarType::Ptr(inner) | VarType::Arr(inner, _) => get_type_size(inner),
        VarType::Fund(_) => unreachable!(), // this function should only be called with a type of pointer or array
    }
}

pub fn get_pointee_type(t: &VarType) -> VarType {
    match t {
        VarType::Ptr(inner) | VarType::Arr(inner, _) => *inner.clone(),
        VarType::Fund(_) => unreachable!(), // this function should only be called with a type of pointer or array
    }
}
