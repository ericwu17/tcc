use std::collections::HashMap;

use crate::{
    parser::{expr_parser::Expr, Statement},
    tac::{get_new_temp_name, Identifier},
};

use super::{
    expr::{generate_expr_tac, ValTarget},
    generate_statement_tac, get_new_label_number, CodeEnv, TacInstr,
};

pub fn generate_continue_tac(code_env: &CodeEnv) -> Vec<TacInstr> {
    match &code_env.loop_label_begin {
        Some(label) => vec![TacInstr::Jmp(label.clone())],
        None => panic!("continue statement outside of loop"),
    }
}

pub fn generate_break_tac(code_env: &CodeEnv) -> Vec<TacInstr> {
    match &code_env.loop_label_end {
        Some(label) => vec![TacInstr::Jmp(label.clone())],
        None => panic!("break statement outside of loop"),
    }
}

pub fn gen_while_loop_tac(
    condition: &Expr,
    body: &Statement,
    code_env: &mut CodeEnv,
) -> Vec<TacInstr> {
    let label_num = get_new_label_number();
    let label_loop_begin = format!("begin_while_{}", label_num);
    let label_loop_end = format!("end_while_{}", label_num);

    let outer_loop_label_end = code_env.loop_label_end.clone();
    let outer_loop_label_begin = code_env.loop_label_begin.clone();

    code_env.loop_label_begin = Some(label_loop_begin.clone());
    code_env.loop_label_end = Some(label_loop_end.clone());

    let mut result = Vec::new();
    result.push(TacInstr::Label(label_loop_begin.clone()));
    let (expr_result, expr_val) = generate_expr_tac(condition, code_env, ValTarget::Generate, None);
    result.extend(expr_result);
    result.push(TacInstr::JmpZero(label_loop_end.clone(), expr_val));
    result.extend(generate_statement_tac(body, code_env));
    result.push(TacInstr::Jmp(label_loop_begin));
    result.push(TacInstr::Label(label_loop_end));

    code_env.loop_label_end = outer_loop_label_end;
    code_env.loop_label_begin = outer_loop_label_begin;

    result
}

pub fn gen_for_loop_tac(
    initial_expr: &Statement,
    control_expr: Option<&Expr>,
    post_expr: Option<&Expr>,
    body: &Statement,
    code_env: &mut CodeEnv,
) -> Vec<TacInstr> {
    let label_num = get_new_label_number();
    let start_loop_label = format!("begin_for_{}", label_num);
    let exit_loop_label = format!("end_for_{}", label_num);
    let before_post_expr_label = format!("before_post_expr_for_{}", label_num);

    let outer_loop_label_end = code_env.loop_label_end.clone();
    let outer_loop_label_begin = code_env.loop_label_begin.clone();

    code_env.loop_label_end = Some(exit_loop_label.clone());
    code_env.loop_label_begin = Some(before_post_expr_label.clone());

    let mut result = Vec::new();
    let mut header_var_map: HashMap<String, Identifier> = HashMap::new();
    match initial_expr {
        Statement::Declare(var_name, optional_expr, t) => {
            let var_temp_loc = get_new_temp_name(t.to_size().unwrap());
            header_var_map.insert(var_name.clone(), var_temp_loc);
            if let Some(expr) = optional_expr {
                let (instrs, _) =
                    generate_expr_tac(expr, code_env, ValTarget::Ident(var_temp_loc), None);
                result.extend(instrs);
            }
        }
        Statement::Expr(expr) => {
            let (instrs, _) = generate_expr_tac(expr, code_env, ValTarget::None, None);
            result.extend(instrs);
        }
        Statement::Empty => {}
        _ => unreachable!(),
    }
    code_env.var_map_list.push(header_var_map);
    result.push(TacInstr::Label(start_loop_label.clone()));

    if let Some(control_expr) = control_expr {
        let (ctrl_instrs, ctrl_val) =
            generate_expr_tac(control_expr, code_env, ValTarget::Generate, None);
        result.extend(ctrl_instrs);
        result.push(TacInstr::JmpZero(exit_loop_label.clone(), ctrl_val));
    }
    result.extend(generate_statement_tac(body, code_env));
    result.push(TacInstr::Label(before_post_expr_label.clone()));
    if let Some(post_exr) = post_expr {
        let (post_instrs, _) = generate_expr_tac(post_exr, code_env, ValTarget::None, None);
        result.extend(post_instrs);
    }
    result.push(TacInstr::Jmp(start_loop_label));
    result.push(TacInstr::Label(exit_loop_label));

    code_env.loop_label_end = outer_loop_label_end;
    code_env.loop_label_begin = outer_loop_label_begin;
    code_env.var_map_list.pop(); // pop off header var map

    result
}
