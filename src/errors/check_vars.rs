use std::collections::HashSet;

use super::display::err_display_no_source;
use crate::parser::{
    expr_parser::{Expr, ExprEnum},
    Program, Statement,
};

/// The check_vars function takes a program AST,
/// and verifies that there are no usages of undeclared variables
/// or doubly-declared variables.
pub fn check_vars(program: &Program) {
    for function in &program.functions {
        let body: &Vec<Statement> = &function.body;
        let mut known_vars = HashSet::new();
        for (arg_name, _) in &function.args {
            known_vars.insert(arg_name.clone());
        }

        check_stmts_vars(body, known_vars);
    }
}

fn check_stmts_vars(stmts: &Vec<Statement>, mut known_var_names: HashSet<String>) {
    // note that known_var_names is a owned hashset, not a reference, because
    // this function add to the hashset, but it should not change the hashset owned
    // by the caller. The caller should clone a known_var_names hashset before passing it
    // into here
    let mut vars_decl_local_scope = HashSet::new();
    for stmt in stmts {
        check_stmt_vars(stmt, &mut known_var_names, &mut vars_decl_local_scope)
    }
}

fn check_stmt_vars(
    stmt: &Statement,
    known_var_names: &mut HashSet<String>,
    vars_decl_local_scope: &mut HashSet<String>,
) {
    match stmt {
        Statement::Continue | Statement::Break | Statement::Empty => {}
        Statement::Return(expr) => check_expr_vars(expr, known_var_names),
        Statement::Declare(var_name, optional_expr, _) => {
            if let Some(expr) = optional_expr {
                check_expr_vars(expr, known_var_names);
            }
            if vars_decl_local_scope.contains(var_name) {
                err_display_no_source(format!("variable declared twice: {}", var_name));
            }
            known_var_names.insert(var_name.clone());
            vars_decl_local_scope.insert(var_name.clone());
        }
        Statement::CompoundStmt(inner_stmts) => {
            check_stmts_vars(inner_stmts, known_var_names.clone())
        }
        Statement::If(ctrl_expr, taken_expr, option_not_taken_expr) => {
            check_expr_vars(ctrl_expr, known_var_names);
            check_stmt_vars(taken_expr, known_var_names, vars_decl_local_scope);
            if let Some(not_taken_expr) = option_not_taken_expr {
                check_stmt_vars(not_taken_expr, known_var_names, vars_decl_local_scope);
            }
        }
        Statement::While(ctrl_expr, body) => {
            check_expr_vars(ctrl_expr, known_var_names);
            check_stmt_vars(body, known_var_names, vars_decl_local_scope);
        }
        Statement::For(init_expr, ctrl_expr, post_expr, body) => check_for_loop_vars(
            init_expr,
            ctrl_expr.as_ref().unwrap_or(&Expr::new(ExprEnum::Int(1))),
            post_expr.as_ref().unwrap_or(&Expr::new(ExprEnum::Int(0))),
            body,
            known_var_names.clone(),
            vars_decl_local_scope,
        ),
        Statement::Expr(expr) => check_expr_vars(expr, known_var_names),
    }
}

fn check_for_loop_vars(
    init_expr: &Statement,
    ctrl_expr: &Expr,
    post_expr: &Expr,
    body: &Statement,
    mut known_var_names: HashSet<String>,
    vars_decl_local_scope: &mut HashSet<String>,
) {
    match init_expr {
        Statement::Declare(var_name, optional_expr, _) => {
            if let Some(decl_expr) = optional_expr {
                check_expr_vars(decl_expr, &known_var_names);
            }
            known_var_names.insert(var_name.clone());
        }

        Statement::Expr(decl_expr) => {
            check_expr_vars(decl_expr, &known_var_names);
        }
        Statement::Empty => {}
        _ => unreachable!(),
    }

    check_expr_vars(ctrl_expr, &known_var_names);
    check_expr_vars(post_expr, &known_var_names);

    check_stmt_vars(body, &mut known_var_names, vars_decl_local_scope);
}

fn check_expr_vars(expr: &Expr, known_var_names: &HashSet<String>) {
    // note that known_var_names is a reference, not a owned hashset, because this
    // function does not modify it.

    let mut var_name_to_check = None;
    let mut exprs_to_check = Vec::new();

    match &expr.content {
        ExprEnum::Int(_) | ExprEnum::StaticStrPtr(_) => {}
        ExprEnum::Var(var_name) => var_name_to_check = Some(var_name),
        ExprEnum::UnOp(_, inner_expr) => exprs_to_check = vec![inner_expr.as_ref()],
        ExprEnum::BinOp(_, expr1, expr2) => exprs_to_check = vec![expr1.as_ref(), expr2.as_ref()],
        ExprEnum::Ternary(expr1, expr2, expr3) => {
            exprs_to_check = vec![expr1.as_ref(), expr2.as_ref(), expr3.as_ref()]
        }
        ExprEnum::FunctionCall(_, exprs) => exprs_to_check = exprs.iter().collect(),
        ExprEnum::Deref(expr) => exprs_to_check = vec![expr],
        ExprEnum::Ref(expr) => exprs_to_check = vec![expr],
        ExprEnum::PostfixDec(var_name) => exprs_to_check = vec![var_name],
        ExprEnum::PostfixInc(var_name) => exprs_to_check = vec![var_name],
        ExprEnum::PrefixDec(var_name) => exprs_to_check = vec![var_name],
        ExprEnum::PrefixInc(var_name) => exprs_to_check = vec![var_name],
        ExprEnum::Sizeof(inner_expr) => exprs_to_check = vec![inner_expr],
        ExprEnum::ArrInitExpr(exprs) => exprs_to_check = exprs.iter().collect(),
    }

    if let Some(var_name) = var_name_to_check {
        if !known_var_names.contains(var_name) {
            err_display_no_source(format!("undeclared variable: {}", var_name));
        }
    }
    for expr in exprs_to_check {
        check_expr_vars(expr, known_var_names);
    }
}
