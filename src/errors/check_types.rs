use std::collections::HashMap;

use crate::parser::{expr_parser::Expr, Program, Statement};
use crate::types::{FundT, VarType};

use super::check_bin_op_exprs::get_binop_type;
use super::display::err_display_no_source;

#[derive(Debug)]
pub struct CodeEnv {
    // a list of maps, one for each scope level, mapping variable names to types
    pub var_map_list: Vec<HashMap<String, VarType>>,
}

impl CodeEnv {
    fn new() -> Self {
        CodeEnv {
            var_map_list: Vec::new(),
        }
    }
}

fn resolve_variable_to_temp_name(name: &String, code_env: &CodeEnv) -> VarType {
    for var_map in code_env.var_map_list.iter().rev() {
        if let Some(name) = var_map.get(name) {
            return name.clone();
        }
    }
    // unreachable because check_vars should have already checked that each variable was declared properly.
    unreachable!()
}

pub fn check_types(program: &mut Program) {
    for function in &mut program.functions {
        let mut code_env = CodeEnv::new();
        let mut this_scopes_variable_map: HashMap<String, VarType> = HashMap::new();

        for (arg_name, arg_type) in &function.args {
            this_scopes_variable_map.insert(arg_name.clone(), arg_type.clone());
        }
        code_env.var_map_list.push(this_scopes_variable_map);

        check_compound_stmt_types(&mut function.body, &mut code_env);
    }
}

pub fn check_compound_stmt_types(stmts: &mut Vec<Statement>, code_env: &mut CodeEnv) {
    let this_scopes_variable_map: HashMap<String, VarType> = HashMap::new();
    code_env.var_map_list.push(this_scopes_variable_map);

    for statement in stmts {
        check_stmt_types(statement, code_env);
    }

    code_env.var_map_list.pop();
}

pub fn check_stmt_types(stmt: &mut Statement, code_env: &mut CodeEnv) {
    match stmt {
        Statement::Continue | Statement::Break | Statement::Empty => {}
        Statement::Return(expr) => {
            let returned_type = get_type(expr, code_env);
            match returned_type {
                Some(VarType::Fund(_)) | None => {}
                _ => {
                    err_display_no_source("return type of function must be a fundamental type");
                }
            }
        }
        Statement::Declare(var_name, optional_expr, expected_type) => {
            if let Some(init_expr) = optional_expr {
                if !are_interchangable_types(
                    &get_type(init_expr, code_env),
                    &Some(expected_type.clone()),
                ) {
                    err_display_no_source(format!(
                        "incompatible types in declaration of {}",
                        var_name
                    ))
                }
            }
            let var_map_list = &mut code_env.var_map_list;
            let last_elem_index = var_map_list.len() - 1;
            let this_scopes_variable_map = var_map_list.get_mut(last_elem_index).unwrap();
            this_scopes_variable_map.insert(var_name.clone(), expected_type.clone());
        }
        Statement::CompoundStmt(stmts) => {
            for stmt in stmts {
                check_stmt_types(stmt, code_env);
            }
        }
        Statement::If(ctrl_expr, taken_branch, not_taken_branch) => {
            check_bool_expr(ctrl_expr, code_env);
            check_stmt_types(taken_branch, code_env);
            if let Some(not_taken_branch) = not_taken_branch {
                check_stmt_types(not_taken_branch, code_env);
            }
        }
        Statement::While(ctrl_expr, body) => {
            check_bool_expr(ctrl_expr, code_env);
            check_stmt_types(body, code_env);
        }
        Statement::For(init_expr, ctrl_expr, post_expr, body) => {
            let this_scopes_variable_map: HashMap<String, VarType> = HashMap::new();
            code_env.var_map_list.push(this_scopes_variable_map);

            check_stmt_types(init_expr, code_env);
            if let Some(ctrl_expr) = ctrl_expr {
                check_bool_expr(ctrl_expr, code_env);
            }
            if let Some(post_expr) = post_expr {
                get_type(post_expr, code_env);
            }
            check_stmt_types(body, code_env);
            code_env.var_map_list.pop();
        }
        Statement::Expr(expr) => {
            get_type(expr, code_env);
        }
    }
}

fn check_bool_expr(expr: &mut Expr, code_env: &CodeEnv) {
    let t = get_type(expr, code_env);
    match t {
        None | Some(VarType::Fund(_)) | Some(VarType::Ptr(_)) => {
            // ok
        }
        other => {
            err_display_no_source(format!(
                "expression is not a boolean value, has type {:?}",
                other
            ));
        }
    }
}

// This function computes the type of an expression, returning None
// if the type is "flexible" such as a literal 3 which can be either an int or a long.
// This function calls err_display_no_source (and thus exits) if it detects improper use as deened by:
//      adding pointer to something not an integer type
//      mixing number/pointer types
//      doing anything with an array which is not "index" or "ref"
//      assignment to something which is not an l_value
pub fn get_type(expr: &mut Expr, code_env: &CodeEnv) -> Option<VarType> {
    match expr {
        Expr::Int(_) => return None,
        Expr::Var(var_name) => return Some(resolve_variable_to_temp_name(var_name, code_env)),
        Expr::UnOp(_, inner) => {
            let inner_type = get_type(inner, code_env);
            match &inner_type {
                Some(t) => match &t {
                    VarType::Fund(_) => {
                        return inner_type;
                    }
                    VarType::Ptr(_) => {
                        err_display_no_source("cannot apply unary operator to pointer");
                    }
                    VarType::Arr(_, _) => {
                        err_display_no_source("cannot apply unary operator to array");
                    }
                },
                None => {
                    return inner_type;
                }
            }
        }
        Expr::BinOp(op, expr_1, expr_2) => get_binop_type(*op, expr_1, expr_2, code_env),
        Expr::Ternary(ctrl_expr, expr_1, expr_2) => {
            match get_type(ctrl_expr, code_env) {
                None | Some(VarType::Fund(_)) => {
                    // ok
                }
                _ => {
                    err_display_no_source(
                        "control expression of ternary must be boolean compatible",
                    );
                }
            }

            let t1 = get_type(expr_1, code_env);
            let t2 = get_type(expr_2, code_env);
            if !are_interchangable_types(&t1, &t2) {
                err_display_no_source("cannot mix types in ternary expression");
            }
            return t1;
        }
        Expr::FunctionCall(_, _) => {
            // we will say, for now, that function calls return a flexible type. (functions may only return fundamental types)
            return None;
        }
        Expr::Deref(inner) => {
            let inner_type = get_type(inner, code_env);
            if let Some(VarType::Ptr(t)) = inner_type {
                return Some(*t);
            } else if let Some(VarType::Arr(t, _)) = inner_type {
                return Some(*t);
            } else {
                err_display_no_source("tried to dereference something that isn't a pointer.")
            }
        }
        Expr::Ref(inner) => {
            if !is_l_value(inner) {
                err_display_no_source("tried to take a reference to something that isn't a lvalue.")
            }
            let inner_type = get_type(inner, code_env).unwrap();

            return Some(VarType::Ptr(Box::new(inner_type)));
        }
        Expr::PostfixDec(inner)
        | Expr::PostfixInc(inner)
        | Expr::PrefixDec(inner)
        | Expr::PrefixInc(inner) => {
            if !is_l_value(inner) {
                err_display_no_source("tried use ++ or -- on something that isn't a lvalue.")
            }
            return Some(get_type(inner, code_env).unwrap());
        }
        Expr::Sizeof(inner_expr) => {
            let inner_type = get_type(inner_expr, code_env);
            let inner_type = inner_type.unwrap_or(VarType::Fund(FundT::Int));
            *expr = Expr::Int(inner_type.num_bytes() as i64);
            return Some(VarType::Fund(crate::types::FundT::Long));
        }
    }
}

pub fn is_l_value(expr: &Expr) -> bool {
    match expr {
        Expr::Int(_) => false,
        Expr::Var(_) => true,
        Expr::UnOp(_, _) => false,
        Expr::BinOp(_, _, _) => false,
        Expr::Ternary(_, _, _) => false,
        Expr::FunctionCall(_, _) => false,
        Expr::Deref(_) => true,
        Expr::Ref(_) => false,
        Expr::PostfixDec(_) => false,
        Expr::PostfixInc(_) => false,
        Expr::PrefixDec(_) => false,
        Expr::PrefixInc(_) => false,
        Expr::Sizeof(_) => false,
    }
}

pub fn are_interchangable_types(t1: &Option<VarType>, t2: &Option<VarType>) -> bool {
    match (t1, t2) {
        (None, None) => true,
        (None, Some(t)) | (Some(t), None) => match t {
            VarType::Fund(_) => true,
            VarType::Ptr(_) => false,
            VarType::Arr(_, _) => false,
        },
        (Some(inner_t1), Some(inner_t2)) => match (inner_t1, inner_t2) {
            (VarType::Fund(_), VarType::Fund(_)) => true,
            (VarType::Fund(_), VarType::Ptr(_)) => false,
            (VarType::Fund(_), VarType::Arr(_, _)) => false,
            (VarType::Ptr(_), VarType::Fund(_)) => false,
            (VarType::Ptr(t1), VarType::Ptr(t2)) => t1 == t2,
            (VarType::Ptr(_), VarType::Arr(_, _)) => false,
            (VarType::Arr(_, _), VarType::Fund(_)) => false,
            (VarType::Arr(_, _), VarType::Ptr(_)) => false,
            (VarType::Arr(t1, l1), VarType::Arr(t2, l2)) => t1 == t2 && l1 == l2,
        },
    }
}
