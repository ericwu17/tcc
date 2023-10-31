use std::collections::HashMap;

use crate::codegen::builtin_functions::BUILTIN_FUNCTIONS;
use crate::parser::expr_parser::ExprEnum;
use crate::parser::{expr_parser::Expr, Program, Statement};
use crate::types::{FundT, VarType};

use super::check_bin_op_exprs::get_binop_type;
use super::display::err_display_no_source;

#[derive(Debug)]
pub struct CodeEnv {
    // a list of maps, one for each scope level, mapping variable names to types
    pub var_map_list: Vec<HashMap<String, VarType>>,
    // a map of function name to the return type of the function
    pub func_ret_type_map: HashMap<String, VarType>,
}

impl CodeEnv {
    fn new(func_ret_type_map: HashMap<String, VarType>) -> Self {
        CodeEnv {
            var_map_list: Vec::new(),
            func_ret_type_map,
        }
    }

    fn get_func_ret_type(&self, func_name: &String) -> VarType {
        if let Some(t) = self.func_ret_type_map.get(func_name) {
            return t.clone();
        }
        for function_decl in BUILTIN_FUNCTIONS {
            if function_decl.name == func_name {
                return function_decl.return_type;
            }
        }

        // unreachble because check_funcs already checked that all functions are already defined
        unreachable!()
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
    let mut func_ret_type_map = HashMap::new();
    for function in &program.functions {
        func_ret_type_map.insert(function.name.clone(), function.return_type.clone());
    }

    for function in &mut program.functions {
        let mut code_env = CodeEnv::new(func_ret_type_map.clone());
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
                Some(VarType::Arr(_, _)) => {
                    err_display_no_source("error: trying to return array from function");
                }
                Some(VarType::Fund(_)) | Some(VarType::Ptr(_)) | None => {}
            }
        }
        Statement::Declare(var_name, optional_expr, expected_type) => {
            if let Some(init_expr) = optional_expr {
                if let ExprEnum::ArrInitExpr(_) = init_expr.content {
                    // ok
                } else if !are_assignment_compatible_types(
                    &Some(expected_type.clone()),
                    &get_type(init_expr, code_env),
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
    let type_;
    match &mut expr.content {
        ExprEnum::Int(_) => {
            type_ = None;
        }
        ExprEnum::Var(var_name) => type_ = Some(resolve_variable_to_temp_name(&var_name, code_env)),
        ExprEnum::UnOp(_, inner) => {
            let inner_type = get_type(inner, code_env);
            match &inner_type {
                Some(t) => match &t {
                    VarType::Fund(_) => {
                        type_ = inner_type;
                    }
                    VarType::Ptr(_) => {
                        err_display_no_source("cannot apply unary operator to pointer");
                    }
                    VarType::Arr(_, _) => {
                        err_display_no_source("cannot apply unary operator to array");
                    }
                },
                None => {
                    type_ = inner_type;
                }
            }
        }
        ExprEnum::BinOp(op, expr_1, expr_2) => {
            type_ = get_binop_type(*op, expr_1, expr_2, code_env);
        }
        ExprEnum::Ternary(ctrl_expr, expr_1, expr_2) => {
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
            type_ = t1;
        }
        ExprEnum::FunctionCall(func_name, exprs) => {
            for expr in exprs {
                get_type(expr, code_env);
            }

            type_ = Some(code_env.get_func_ret_type(func_name));
        }
        ExprEnum::Deref(inner) => {
            let inner_type = get_type(inner, code_env);
            if let Some(VarType::Ptr(t)) = inner_type {
                type_ = Some(*t);
            } else if let Some(VarType::Arr(t, _)) = inner_type {
                type_ = Some(*t);
            } else {
                err_display_no_source("tried to dereference something that isn't a pointer.")
            }
        }
        ExprEnum::Ref(inner) => {
            if !is_l_value(&inner) {
                err_display_no_source("tried to take a reference to something that isn't a lvalue.")
            }
            let inner_type = get_type(inner, code_env).unwrap();

            type_ = Some(VarType::Ptr(Box::new(inner_type)));
        }
        ExprEnum::PostfixDec(inner)
        | ExprEnum::PostfixInc(inner)
        | ExprEnum::PrefixDec(inner)
        | ExprEnum::PrefixInc(inner) => {
            if !is_l_value(&inner) {
                err_display_no_source("tried use ++ or -- on something that isn't a lvalue.")
            }
            type_ = Some(get_type(inner, code_env).unwrap());
        }
        ExprEnum::Sizeof(inner_expr) => {
            let inner_type = get_type(inner_expr, code_env);
            let inner_type = inner_type.unwrap_or(VarType::Fund(FundT::Int));
            *expr = Expr::new(ExprEnum::Int(inner_type.num_bytes() as i64));
            type_ = Some(VarType::Fund(crate::types::FundT::Long));
        }
        ExprEnum::ArrInitExpr(_) => type_ = None,
        ExprEnum::StaticStrPtr(_) => {
            type_ = Some(VarType::Ptr(Box::new(VarType::Fund(FundT::Char))));
        }
    };
    expr.type_ = type_.clone();
    return type_;
}

pub fn is_l_value(expr: &Expr) -> bool {
    match expr.content {
        ExprEnum::Var(_) | ExprEnum::Deref(_) => true,
        ExprEnum::Int(_)
        | ExprEnum::BinOp(_, _, _)
        | ExprEnum::UnOp(_, _)
        | ExprEnum::Ternary(_, _, _)
        | ExprEnum::FunctionCall(_, _)
        | ExprEnum::Ref(_)
        | ExprEnum::PostfixDec(_)
        | ExprEnum::PostfixInc(_)
        | ExprEnum::PrefixDec(_)
        | ExprEnum::PrefixInc(_)
        | ExprEnum::Sizeof(_)
        | ExprEnum::ArrInitExpr(_)
        | ExprEnum::StaticStrPtr(_) => false,
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

pub fn are_assignment_compatible_types(t1: &Option<VarType>, t2: &Option<VarType>) -> bool {
    // trying to assign t1 to t2
    match (t1, t2) {
        (None, None) => true,
        (None, Some(t)) | (Some(t), None) => match t {
            VarType::Fund(_) => true,
            VarType::Ptr(_) => false,
            VarType::Arr(_, _) => false,
        },
        (Some(inner_t1), Some(inner_t2)) => match (inner_t1, inner_t2) {
            (VarType::Fund(_), VarType::Fund(_))
            | (VarType::Ptr(_), VarType::Ptr(_))
            | (VarType::Ptr(_), VarType::Arr(_, _)) => true,
            (VarType::Fund(_), VarType::Ptr(_))
            | (VarType::Fund(_), VarType::Arr(_, _))
            | (VarType::Ptr(_), VarType::Fund(_))
            | (VarType::Arr(_, _), VarType::Fund(_))
            | (VarType::Arr(_, _), VarType::Ptr(_)) => false,
            (VarType::Arr(_, _), VarType::Arr(_, _)) => false,
        },
    }
}
