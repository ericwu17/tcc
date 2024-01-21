use crate::parser::expr_parser::ExprEnum;

use super::{
    expr_parser::{BinOp, Expr, UnOp},
    Program, Statement,
};

/// evaluates constant expressions in a program AST.
/// for example, the expression (-(3+7) * 5) will be replaced with -50, since
/// the expression consists of only integers that can be simplified at compile time.
pub fn eval_program_const_exprs(program: &mut Program) {
    for function in &mut program.functions {
        eval_compound_stmt_exprs(&mut function.body);
    }
}

fn eval_compound_stmt_exprs(stmts: &mut Vec<Statement>) {
    for statement in stmts {
        eval_stmt_exprs(statement);
    }
}

fn eval_stmt_exprs(stmt: &mut Statement) {
    let mut exprs_to_eval = Vec::new();
    match stmt {
        Statement::Continue | Statement::Empty | Statement::Break => {}
        Statement::Return(expr) | Statement::Expr(expr) => {
            exprs_to_eval = vec![expr];
        }
        Statement::Declare(_, optional_expr, _) => {
            if let Some(expr) = optional_expr {
                exprs_to_eval = vec![expr];
            }
        }
        Statement::CompoundStmt(stmts) => {
            eval_compound_stmt_exprs(stmts);
        }
        Statement::If(expr, taken_stmt, opt_non_taken_stmt) => {
            exprs_to_eval = vec![expr];
            eval_stmt_exprs(taken_stmt);
            if let Some(not_taken_stmt) = opt_non_taken_stmt {
                eval_stmt_exprs(not_taken_stmt);
            }
        }
        Statement::While(expr, body_stmt) => {
            exprs_to_eval = vec![expr];
            eval_stmt_exprs(body_stmt);
        }
        Statement::For(init_stmt, ctrl_expr, post_stmt, body_stmt) => {
            if let Some(ctrl_expr) = ctrl_expr {
                exprs_to_eval.push(ctrl_expr);
            }
            if let Some(post_stmt) = post_stmt {
                exprs_to_eval.push(post_stmt);
            }
            eval_stmt_exprs(init_stmt);
            eval_stmt_exprs(body_stmt);
        }
    }

    for expr in exprs_to_eval {
        eval_expr(expr);
    }
}

fn eval_expr(expr: &mut Expr) {
    match &mut expr.content {
        ExprEnum::BinOp(op, expr_1, expr_2) => {
            eval_expr(expr_1);
            eval_expr(expr_2);
            if let Some(simplified_expr) = eval_binop(*op, *expr_1.to_owned(), *expr_2.to_owned()) {
                *expr = simplified_expr;
            }
        }
        ExprEnum::UnOp(op, inner_expr) => {
            eval_expr(inner_expr);
            if let Some(simplified_expr) = eval_unop(*op, *inner_expr.to_owned()) {
                *expr = simplified_expr;
            }
        }
        ExprEnum::Ternary(_, _, _) => {}

        ExprEnum::FunctionCall(_, args) => {
            for arg in args {
                eval_expr(arg);
            }
        }
        ExprEnum::ArrInitExpr(exprs) => {
            for expr in exprs {
                eval_expr(expr);
            }
        }

        ExprEnum::Int(_)
        | ExprEnum::Var(_)
        | ExprEnum::Deref(_)
        | ExprEnum::Ref(_)
        | ExprEnum::PostfixDec(_)
        | ExprEnum::PostfixInc(_)
        | ExprEnum::PrefixDec(_)
        | ExprEnum::PrefixInc(_)
        | ExprEnum::Sizeof(_)
        | ExprEnum::StaticStrPtr(_) => {}
    }
}

fn eval_unop(op: UnOp, expr: Expr) -> Option<Expr> {
    let val: i64 = match expr.content {
        ExprEnum::Int(v) => v,
        _ => return None,
    };

    let new_val;
    match op {
        UnOp::Negation => {
            new_val = -val;
        }
        UnOp::BitwiseComplement => new_val = !val,
        UnOp::Not => {
            new_val = if val == 0 { 1 } else { 0 };
        }
    }

    Some(Expr {
        content: ExprEnum::Int(new_val),
        type_: expr.type_,
    })
}

fn eval_binop(op: BinOp, expr_1: Expr, expr_2: Expr) -> Option<Expr> {
    let val_1: i64 = match expr_1.content {
        ExprEnum::Int(v) => v,
        _ => return None,
    };
    let val_2: i64 = match expr_2.content {
        ExprEnum::Int(v) => v,
        _ => return None,
    };

    let new_val = match op {
        BinOp::Multiply => val_1 * val_2,
        BinOp::Divide => val_1 / val_2,
        BinOp::Modulus => val_1 % val_2,
        BinOp::Plus => val_1 + val_2,
        BinOp::Minus => val_1 - val_2,
        BinOp::GreaterThan => bool_to_i64(val_1 > val_2),
        BinOp::GreaterThanEq => bool_to_i64(val_1 >= val_2),
        BinOp::LessThan => bool_to_i64(val_1 < val_2),
        BinOp::LessThanEq => bool_to_i64(val_1 <= val_2),
        BinOp::Equals => bool_to_i64(val_1 == val_2),
        BinOp::NotEquals => bool_to_i64(val_1 != val_2),
        BinOp::LogicalAnd => bool_to_i64(i64_to_bool(val_1) && i64_to_bool(val_2)),
        BinOp::LogicalOr => bool_to_i64(i64_to_bool(val_1) || i64_to_bool(val_2)),
        BinOp::Assign => return None,
    };

    Some(Expr {
        content: ExprEnum::Int(new_val),
        type_: None,
    })
}

fn bool_to_i64(b: bool) -> i64 {
    if b {
        1
    } else {
        0
    }
}

fn i64_to_bool(x: i64) -> bool {
    if x == 0 {
        false
    } else {
        true
    }
}
