use std::collections::HashMap;

use crate::parser::expr_parser::{BinOp, Expr, UnOp};

use super::{get_new_label, resolve_variable, X86Instruction, X86Routine};

pub fn generate_expr_code(expr: &Expr, var_map_list: &Vec<HashMap<String, usize>>) -> X86Routine {
    match expr {
        Expr::Var(var_name) => {
            let var_location = format!("[rbp - {}]", resolve_variable(var_name, var_map_list));
            let mut result = X86Routine::new();
            result.push(X86Instruction::double_op_instruction(
                "mov",
                "rdi",
                &var_location,
            ));
            result.push(X86Instruction::single_op_instruction("push", "rdi"));
            return result;
        }
        Expr::Assign(var_name, expr) => {
            let var_location = format!("[rbp - {}]", resolve_variable(&var_name, var_map_list));

            let mut result = generate_expr_code(expr, var_map_list);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction(
                "mov",
                &var_location,
                "rdi",
            ));
            result.push(X86Instruction::single_op_instruction("push", "rdi"));
            return result;
        }
        Expr::Int(v) => {
            let operand = format!("{}", v);
            return X86Routine::single_instruction("push", vec![&operand]);
        }
        Expr::UnOp(op, inner_expr) => {
            let action: X86Routine = match op {
                UnOp::Negation => X86Routine::single_instruction("neg", vec!["rdi"]),
                UnOp::BitwiseComplement => X86Routine::single_instruction("not", vec!["rdi"]),
                UnOp::Not => X86Routine {
                    instructions: vec![
                        X86Instruction::double_op_instruction("cmp", "rdi", "0"),
                        X86Instruction::double_op_instruction("mov", "rdi", "0"),
                        X86Instruction::single_op_instruction("sete", "dil"),
                    ],
                },
            };

            let mut code = generate_expr_code(inner_expr, var_map_list);
            code.push(X86Instruction::single_op_instruction("pop", "rdi"));
            code.extend(action);
            code.push(X86Instruction::single_op_instruction("push", "rdi"));
            return code;
        }
        Expr::BinOp(op, expr1, expr2) => {
            if op == &BinOp::LogicalAnd || op == &BinOp::LogicalOr {
                return generate_short_circuiting_binop_code(op, expr1, expr2, var_map_list);
            }

            let expr_1_code = generate_expr_code(expr1, var_map_list);
            let expr_2_code = generate_expr_code(expr2, var_map_list);

            let mut code = X86Routine::new();
            code.extend(expr_1_code);
            code.extend(expr_2_code);

            code.extend(generate_binop_code(op));

            return code;
        }
        Expr::Ternary(decision_expr, expr1, expr2) => {
            let mut result = generate_expr_code(decision_expr, var_map_list);

            let label_1 = get_new_label();
            let label_end = get_new_label();
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            result.push(X86Instruction::single_op_instruction("je", &label_1));
            result.extend(generate_expr_code(expr1, var_map_list));
            result.push(X86Instruction::single_op_instruction("jmp", &label_end));
            result.push(X86Instruction {
                operation: label_1,
                operands: vec![],
            });
            result.extend(generate_expr_code(expr2, var_map_list));
            result.push(X86Instruction {
                operation: label_end,
                operands: vec![],
            });

            return result;
        }
    }
}

fn generate_binop_code(op: &BinOp) -> X86Routine {
    let mut code = X86Routine::new();
    code.push(X86Instruction::single_op_instruction("pop", "rsi")); // expr 2 in rsi
    code.push(X86Instruction::single_op_instruction("pop", "rdi")); // expr 1 in rdi

    match op {
        BinOp::Plus => code.push(X86Instruction::double_op_instruction("add", "rdi", "rsi")),
        BinOp::Minus => code.push(X86Instruction::double_op_instruction("sub", "rdi", "rsi")),
        BinOp::Multiply => code.push(X86Instruction::double_op_instruction("imul", "rdi", "rsi")),
        BinOp::Divide => {
            code.push(X86Instruction::double_op_instruction("mov", "eax", "edi"));
            code.push(X86Instruction::no_operands_instr("cdq"));
            code.push(X86Instruction::single_op_instruction("idiv", "esi"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "rax"));
        }
        BinOp::Modulus => {
            code.push(X86Instruction::double_op_instruction("mov", "eax", "edi"));
            code.push(X86Instruction::no_operands_instr("cdq"));
            code.push(X86Instruction::single_op_instruction("idiv", "esi"));
            // the remainder is stored in edx.
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "rdx"));
        }
        BinOp::LogicalOr => {
            // should have used the generate_short_circuiting_binop_code() function
            unreachable!();
        }
        BinOp::LogicalAnd => {
            // should have used the generate_short_circuiting_binop_code() function
            unreachable!();
        }
        BinOp::Equals => {
            code.push(X86Instruction::double_op_instruction("cmp", "rdi", "rsi"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "0"));
            code.push(X86Instruction::single_op_instruction("sete", "dil"));
        }
        BinOp::NotEquals => {
            code.push(X86Instruction::double_op_instruction("cmp", "rdi", "rsi"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "0"));
            code.push(X86Instruction::single_op_instruction("setne", "dil"));
        }
        BinOp::GreaterThan => {
            code.push(X86Instruction::double_op_instruction("cmp", "rdi", "rsi"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "0"));
            code.push(X86Instruction::single_op_instruction("setg", "dil"));
        }
        BinOp::LessThan => {
            code.push(X86Instruction::double_op_instruction("cmp", "rdi", "rsi"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "0"));
            code.push(X86Instruction::single_op_instruction("setl", "dil"));
        }
        BinOp::GreaterThanEq => {
            code.push(X86Instruction::double_op_instruction("cmp", "rdi", "rsi"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "0"));
            code.push(X86Instruction::single_op_instruction("setge", "dil"));
        }
        BinOp::LessThanEq => {
            code.push(X86Instruction::double_op_instruction("cmp", "rdi", "rsi"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "0"));
            code.push(X86Instruction::single_op_instruction("setle", "dil"));
        }
    }

    // final result goes into rdi
    code.push(X86Instruction::single_op_instruction("push", "rdi"));
    return code;
}

fn generate_short_circuiting_binop_code(
    op: &BinOp,
    expr1: &Expr,
    expr2: &Expr,
    var_map_list: &Vec<HashMap<String, usize>>,
) -> X86Routine {
    match op {
        BinOp::LogicalAnd => {
            let label1 = get_new_label();
            let label2 = get_new_label();

            let mut result = generate_expr_code(expr1, var_map_list);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            result.push(X86Instruction::single_op_instruction("je", &label1));

            result.extend(generate_expr_code(expr2, var_map_list));

            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            result.push(X86Instruction::single_op_instruction("setne", "al"));
            result.push(X86Instruction::single_op_instruction("push", "rax"));
            result.push(X86Instruction::single_op_instruction("jmp", &label2));
            result.push(X86Instruction {
                operation: label1,
                operands: vec![],
            });
            result.push(X86Instruction::single_op_instruction("push", "0"));
            result.push(X86Instruction {
                operation: label2,
                operands: vec![],
            });

            return result;
        }
        BinOp::LogicalOr => {
            let label1 = get_new_label();
            let label2 = get_new_label();

            let mut result = generate_expr_code(expr1, var_map_list);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            result.push(X86Instruction::single_op_instruction("jne", &label1));

            result.extend(generate_expr_code(expr2, var_map_list));

            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            result.push(X86Instruction::single_op_instruction("setne", "al"));
            result.push(X86Instruction::single_op_instruction("push", "rax"));
            result.push(X86Instruction::single_op_instruction("jmp", &label2));
            result.push(X86Instruction {
                operation: label1,
                operands: vec![],
            });
            result.push(X86Instruction::single_op_instruction("push", "1"));
            result.push(X86Instruction {
                operation: label2,
                operands: vec![],
            });

            return result;
        }
        _ => unreachable!(),
    }
}
