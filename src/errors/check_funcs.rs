use super::display::err_display_no_source;
use crate::{
    codegen::builtin_functions::BUILTIN_FUNCTIONS,
    parser::{
        expr_parser::{Expr, ExprEnum},
        Program, Statement,
    },
};

#[derive(PartialEq, Eq)]
struct FuncDecl {
    name: String,
    num_args: usize,
}

// The check_funcs function takes a program AST,
// and verifies that:
//    - Every function call must have a definition with the same number of parameters
//    - There are no duplicate function names
//    - There is a "main" function
pub fn check_funcs(program: &Program) {
    let mut known_functions = Vec::new();
    let mut declared_names = Vec::new();

    for function in &program.functions {
        if declared_names.contains(&function.name) {
            err_display_no_source(format!(
                "function declared more than once: {}",
                function.name
            ));
        }

        known_functions.push(FuncDecl {
            name: function.name.clone(),
            num_args: function.args.len(),
        });
        declared_names.push(function.name.clone());
    }

    if !declared_names.contains(&"main".to_owned()) {
        err_display_no_source("could not find \"main\" function!");
    }
    for func in &known_functions {
        if func.name == "main" && func.num_args != 0 {
            err_display_no_source(format!(
                "main function must take 0 arguments instead of {}.",
                func.num_args
            ));
        }
    }

    for function in &program.functions {
        let body = &function.body;
        check_stmts_funcs(body, &known_functions);
    }
}

fn check_stmts_funcs(stmts: &Vec<Statement>, known_funcs: &Vec<FuncDecl>) {
    // note that known_var_names is a owned hashset, not a reference, because
    // this function add to the hashset, but it should not change the hashset owned
    // by the caller. The caller should clone a known_var_names hashset before passing it
    // into here
    for stmt in stmts {
        check_stmt_funcs(stmt, known_funcs);
    }
}

fn check_stmt_funcs(stmt: &Statement, known_funcs: &Vec<FuncDecl>) {
    match stmt {
        Statement::Continue | Statement::Break | Statement::Empty => {}
        Statement::Return(expr) => check_expr_funcs(expr, known_funcs),
        Statement::Declare(_, optional_expr, _) => {
            if let Some(expr) = optional_expr {
                check_expr_funcs(expr, known_funcs);
            }
        }
        Statement::CompoundStmt(inner_stmts) => check_stmts_funcs(inner_stmts, known_funcs),
        Statement::If(ctrl_expr, taken_expr, option_not_taken_expr) => {
            check_expr_funcs(ctrl_expr, known_funcs);
            check_stmt_funcs(taken_expr, known_funcs);
            if let Some(not_taken_expr) = option_not_taken_expr {
                check_stmt_funcs(not_taken_expr, known_funcs);
            }
        }
        Statement::While(ctrl_expr, body) => {
            check_expr_funcs(ctrl_expr, known_funcs);
            check_stmt_funcs(body, known_funcs);
        }
        Statement::For(init_expr, ctrl_expr, post_expr, body) => {
            check_stmt_funcs(init_expr, known_funcs);
            if let Some(expr) = ctrl_expr {
                check_expr_funcs(expr, known_funcs);
            }
            if let Some(expr) = post_expr {
                check_expr_funcs(expr, known_funcs);
            }
            check_stmt_funcs(body, known_funcs);
        }
        Statement::Expr(expr) => check_expr_funcs(expr, known_funcs),
    }
}

fn check_expr_funcs(expr: &Expr, known_funcs: &Vec<FuncDecl>) {
    let mut func_to_check = None;
    let mut exprs_to_check = Vec::new();

    match &expr.content {
        ExprEnum::Int(_)
        | ExprEnum::Var(_)
        | ExprEnum::PostfixDec(_)
        | ExprEnum::PostfixInc(_)
        | ExprEnum::PrefixDec(_)
        | ExprEnum::PrefixInc(_)
        | ExprEnum::StaticStrPtr(_) => {}
        ExprEnum::UnOp(_, inner_expr) => exprs_to_check = vec![inner_expr.as_ref()],
        ExprEnum::BinOp(_, expr1, expr2) => exprs_to_check = vec![expr1.as_ref(), expr2.as_ref()],
        ExprEnum::Ternary(expr1, expr2, expr3) => {
            exprs_to_check = vec![expr1.as_ref(), expr2.as_ref(), expr3.as_ref()]
        }
        ExprEnum::FunctionCall(func_name, exprs) => {
            exprs_to_check = exprs.iter().collect();
            func_to_check = Some(FuncDecl {
                name: func_name.clone(),
                num_args: exprs.len(),
            });
        }
        ExprEnum::Deref(inner_expr) | ExprEnum::Ref(inner_expr) | ExprEnum::Sizeof(inner_expr) => {
            exprs_to_check = vec![inner_expr.as_ref()]
        }
        ExprEnum::ArrInitExpr(exprs) => exprs_to_check = exprs.iter().collect(),
    }

    if let Some(func) = func_to_check {
        if !known_funcs.contains(&func) {
            let mut is_builtin = false;

            for func_decl in BUILTIN_FUNCTIONS {
                if func_decl.name == func.name {
                    is_builtin = true;
                    if func.num_args != func_decl.num_args {
                        err_display_no_source(format!(
                            "{} expects exactly one argument, {} given",
                            func_decl.name, func.num_args,
                        ))
                    }
                }
            }

            if !is_builtin {
                err_display_no_source(format!("undefined function: {}", &func.name))
            }
        }
    }

    for expr in exprs_to_check {
        check_expr_funcs(expr, known_funcs);
    }
}
