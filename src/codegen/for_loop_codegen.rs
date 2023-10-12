use crate::codegen::expr_codegen::generate_expr_code;
use crate::codegen::{
    count_stmt_variable_decls, generate_statement_code, get_new_label, resolve_variable, CodeEnv,
    X86Instruction, X86Routine,
};
use crate::parser::Statement;
use std::collections::HashMap;

pub fn generate_for_loop_code(for_loop: &Statement, code_env: &mut CodeEnv) -> X86Routine {
    match for_loop {
        Statement::For(initial_clause, ctrl_expr, post_expr, body) => {
            let mut this_scopes_variable_map: HashMap<String, usize> = HashMap::new();
            if let Statement::Declare(var_name, _) = initial_clause.as_ref() {
                this_scopes_variable_map.insert(var_name.clone(), code_env.var_index);
                code_env.var_index += 1;
            }
            code_env.var_map_list.push(this_scopes_variable_map);

            let start_loop_label = get_new_label();
            let exit_loop_label = get_new_label();
            let before_post_expr_label = get_new_label();
            code_env.loop_label_end = Some(exit_loop_label.clone());
            code_env.loop_label_begin = Some(before_post_expr_label.clone());

            let mut result = X86Routine::new();
            result.extend(generate_initial_statement_code(
                initial_clause.as_ref(),
                code_env,
            ));
            result.push(X86Instruction::no_operands_instr(&start_loop_label));
            result.extend(generate_ctrl_expr_code(ctrl_expr.as_ref(), code_env));
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            result.push(X86Instruction::single_op_instruction(
                "je",
                &exit_loop_label,
            ));

            result.extend(generate_statement_code(body.as_ref(), code_env));

            result.push(X86Instruction::no_operands_instr(&before_post_expr_label));
            result.extend(generate_post_expr_code(post_expr.as_ref(), code_env));
            result.push(X86Instruction::single_op_instruction(
                "jmp",
                &start_loop_label,
            ));
            result.push(X86Instruction::no_operands_instr(&exit_loop_label));

            code_env.loop_label_end = None;
            code_env.loop_label_begin = None;
            code_env.var_map_list.pop();

            return result;
        }
        _ => unreachable!(),
    }
}

fn generate_post_expr_code(statement: &Statement, code_env: &CodeEnv) -> X86Routine {
    match statement {
        Statement::Empty => {
            return X86Routine::new();
        }
        Statement::Expr(expr) => {
            let mut result = generate_expr_code(expr, &code_env.var_map_list);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            return result;
        }
        _ => unreachable!(),
    }
}

fn generate_ctrl_expr_code(statement: &Statement, code_env: &CodeEnv) -> X86Routine {
    match statement {
        Statement::Empty => {
            return X86Routine::single_instruction("push", vec!["1"]);
        }
        Statement::Expr(expr) => {
            return generate_expr_code(expr, &code_env.var_map_list);
        }
        _ => unreachable!(),
    }
}

fn generate_initial_statement_code(statement: &Statement, code_env: &CodeEnv) -> X86Routine {
    match statement {
        Statement::Declare(var_name, opt_value) => {
            match opt_value {
                Some(expr) => {
                    let var_location = format!(
                        "[rbp - {}]",
                        resolve_variable(&var_name, &code_env.var_map_list)
                    );

                    let mut result = generate_expr_code(expr, &code_env.var_map_list);
                    result.push(X86Instruction::single_op_instruction("pop", "rdi"));
                    result.push(X86Instruction::double_op_instruction(
                        "mov",
                        &var_location,
                        "rdi",
                    ));
                    return result;
                }
                None => unreachable!(),
            };
        }
        Statement::Expr(expr) => {
            // generate the code, pop the value off the stack, and do nothing.
            let mut result = generate_expr_code(&expr, &code_env.var_map_list);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            return result;
        }
        Statement::Empty => {
            return X86Routine::new();
        }
        _ => unreachable!(),
    }
}

pub fn count_for_loop_decls(stmt: &Statement) -> usize {
    match stmt {
        Statement::For(optional_decl, _, _, body) => {
            let mut result = 0;
            if let Statement::Declare(..) = **optional_decl {
                result += 1;
            }
            result += count_stmt_variable_decls(body);

            return result;
        }
        _ => unreachable!(),
    }
}
