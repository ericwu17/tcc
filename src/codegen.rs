use std::collections::HashMap;

use crate::parser::BinOp;
use crate::parser::Expr;
use crate::parser::Program;
use crate::parser::Statement;
use crate::parser::UnOp;

struct X86Routine {
    instructions: Vec<X86Instruction>,
}

static mut LABEL_COUNT: usize = 0;

impl X86Routine {
    fn new() -> Self {
        X86Routine {
            instructions: Vec::new(),
        }
    }

    fn push(&mut self, instr: X86Instruction) {
        self.instructions.push(instr);
    }

    fn extend(&mut self, instrs: X86Routine) {
        self.instructions.extend(instrs.instructions);
    }

    fn to_asm_code(&self) -> String {
        let indent = "  ";
        let mut result = String::new();
        for instr in &self.instructions {
            if !instr.operation.starts_with(".") {
                // we indent everything except for labels
                result.push_str(indent);
                result.push_str(&instr.to_asm_code());
                result.push('\n');
            } else {
                result.push_str(&instr.to_asm_code());
                result.push(':');
                result.push('\n');
            }
        }

        return result;
    }

    fn single_instruction(operation: &'static str, operands: Vec<&'static str>) -> Self {
        X86Routine {
            instructions: vec![X86Instruction {
                operation,
                operands,
            }],
        }
    }
}

struct X86Instruction {
    operation: &'static str,
    operands: Vec<&'static str>,
}

impl X86Instruction {
    fn to_asm_code(&self) -> String {
        let mut result = String::new();
        result.push_str(self.operation);
        result.push(' ');
        for operand in &self.operands {
            result.push_str(operand);
            result.push_str(", ");
        }
        return result;
    }
    fn single_op_instruction(operation: &'static str, operand: &'static str) -> Self {
        X86Instruction {
            operation,
            operands: vec![operand],
        }
    }
    fn double_op_instruction(
        operation: &'static str,
        operand1: &'static str,
        operand2: &'static str,
    ) -> Self {
        X86Instruction {
            operation,
            operands: vec![operand1, operand2],
        }
    }
}

pub fn generate_code(program: Program) -> String {
    let mut result = String::new();
    result.push_str("global _start\n");
    result.push_str("_start:\n");

    assert!(program.function.name == "main");

    let mut var_index: usize = 1; // this is the offset, in units of 8-bytes, of the memory location of the next variable from rbp.
    let routine =
        generate_compound_stmt_code(&program.function.body, true, &mut var_index, &Vec::new());
    result.push_str(&routine.to_asm_code());

    return result;
}

fn generate_compound_stmt_code(
    stmts: &Vec<Statement>,
    is_function_body: bool,
    curr_var_index: &mut usize,
    curr_var_map_list: &Vec<HashMap<String, usize>>,
) -> X86Routine {
    // is_function_body is true if this compound statement is the body of a function.
    // this flag is needed to conditionally insert function prologue and epilogue
    let mut result = X86Routine::new();

    if is_function_body {
        // FUNCTION PROLOGUE
        result.push(X86Instruction::single_op_instruction("push", "rbp"));
        result.push(X86Instruction::double_op_instruction("mov", "rbp", "rsp")); // rbp now points to base of stack frame, and will remain pointing there for the rest of the function

        // every variable gets 8 bytes of space, so we allocate it here
        let space_needed = count_variable_decls(&stmts) * 8;
        result.push(X86Instruction::double_op_instruction(
            "add",
            "rsp",
            Box::leak(format!("{}", space_needed).into_boxed_str()),
        ));
    }

    let this_scopes_variable_map: HashMap<String, usize> = HashMap::new();
    let mut new_var_map_list = curr_var_map_list.clone();
    new_var_map_list.push(this_scopes_variable_map);
    for statement in stmts {
        result.extend(generate_statement_code(
            statement,
            &mut new_var_map_list,
            curr_var_index,
        ));
    }

    if is_function_body {
        let mut need_to_insert_ret_0 = is_function_body;
        if !stmts.is_empty() {
            if let Statement::Return(_) = stmts.get(stmts.len() - 1).unwrap() {
                need_to_insert_ret_0 = false;
            }
        }
        if need_to_insert_ret_0 {
            result.extend(generate_statement_code(
                &Statement::Return(Expr::Int(0)),
                &mut new_var_map_list,
                curr_var_index,
            ));
        }

        // FUNCTION EPILOGUE
        result.push(X86Instruction::double_op_instruction("mov", "rsp", "rbp")); // restore rsp to what it was before this function was called
        result.push(X86Instruction::single_op_instruction("pop", "rbp")); // rbp now points to base of stack frame of outer function
        result.push(X86Instruction {
            operation: "ret",
            operands: vec![],
        });
    }
    return result;
}

fn count_variable_decls(stmts: &Vec<Statement>) -> usize {
    let mut count = 0;
    for stmt in stmts {
        match stmt {
            Statement::CompoundStmt(inner_stmts) => {
                count += count_variable_decls(inner_stmts);
            }
            Statement::Declare(_, _) => {
                count += 1;
            }
            _ => {}
        }
    }
    count
}

fn resolve_variable(
    var_name: &String,
    curr_variable_map_list: &Vec<HashMap<String, usize>>,
) -> &'static str {
    // go through most local scopes first
    for var_map in curr_variable_map_list.iter().rev() {
        if let Some(offset) = var_map.get(var_name) {
            return Box::leak(format!("{}", *offset * 8).into_boxed_str());
        }
    }
    panic!("undeclared variable: {}", var_name);
}

fn generate_statement_code(
    statement: &Statement,
    var_map_list: &mut Vec<HashMap<String, usize>>,
    curr_var_index: &mut usize,
) -> X86Routine {
    match statement {
        Statement::Return(expr) => {
            let mut result = generate_expr_code(expr, var_map_list);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("mov", "rax", "60"));
            result.push(X86Instruction {
                operation: "syscall",
                operands: vec![],
            });
            return result;
        }
        Statement::Declare(var_name, opt_value) => {
            if let Statement::Declare(var_name, _) = statement {
                // let offset = Box::leak(format!("{}", *curr_var_index * 8).into_boxed_str());
                let last_elem_index = var_map_list.len() - 1;
                let this_scopes_variable_map = var_map_list.get_mut(last_elem_index).unwrap();
                if this_scopes_variable_map.get(var_name).is_some() {
                    panic!("doubly declared variable: {}", var_name);
                }

                this_scopes_variable_map.insert(var_name.clone(), *curr_var_index);
                *curr_var_index += 1;
            }

            match opt_value {
                Some(expr) => {
                    let var_location = Box::leak(
                        format!("[rbp - {}]", resolve_variable(var_name, var_map_list))
                            .into_boxed_str(),
                    );

                    let mut result = generate_expr_code(expr, var_map_list);
                    result.push(X86Instruction::single_op_instruction("pop", "rdi"));
                    result.push(X86Instruction::double_op_instruction(
                        "mov",
                        var_location,
                        "rdi",
                    ));
                    return result;
                }
                None => {
                    return X86Routine::new();
                }
            };
        }
        Statement::Expr(expr) => {
            // generate the code, pop the value off the stack, and do nothing.
            let mut result = generate_expr_code(expr, var_map_list);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            return result;
        }
        Statement::CompoundStmt(stmts) => {
            let result = generate_compound_stmt_code(stmts, false, curr_var_index, var_map_list);
            return result;
        }
    }
}

fn generate_expr_code(expr: &Expr, var_map_list: &Vec<HashMap<String, usize>>) -> X86Routine {
    match expr {
        Expr::Var(var_name) => {
            let var_location = Box::leak(
                format!("[rbp - {}]", resolve_variable(var_name, var_map_list)).into_boxed_str(),
            );
            let mut result = X86Routine::new();
            result.push(X86Instruction::double_op_instruction(
                "mov",
                "rdi",
                var_location,
            ));
            result.push(X86Instruction::single_op_instruction("push", "rdi"));
            return result;
        }
        Expr::Assign(var_name, expr) => {
            let var_location = Box::leak(
                format!("[rbp - {}]", resolve_variable(&var_name, var_map_list)).into_boxed_str(),
            );

            let mut result = generate_expr_code(expr, var_map_list);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction(
                "mov",
                var_location,
                "rdi",
            ));
            result.push(X86Instruction::single_op_instruction("push", "rdi"));
            return result;
        }
        Expr::Int(v) => {
            let operand = Box::leak(format!("{}", v).into_boxed_str());
            return X86Routine::single_instruction("push", vec![operand]);
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
            result.push(X86Instruction::single_op_instruction("je", label_1));
            result.extend(generate_expr_code(expr1, var_map_list));
            result.push(X86Instruction::single_op_instruction("jmp", label_end));
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
            code.push(X86Instruction {
                operation: "cdq",
                operands: vec![],
            });
            code.push(X86Instruction::single_op_instruction("idiv", "esi"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "rax"));
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
    code.push(X86Instruction {
        operation: "push",
        operands: vec!["rdi"],
    });
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
            result.push(X86Instruction::single_op_instruction("je", label1));

            result.extend(generate_expr_code(expr2, var_map_list));

            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            result.push(X86Instruction::single_op_instruction("setne", "al"));
            result.push(X86Instruction::single_op_instruction("push", "rax"));
            result.push(X86Instruction::single_op_instruction("jmp", label2));
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
            result.push(X86Instruction::single_op_instruction("jne", label1));

            result.extend(generate_expr_code(expr2, var_map_list));

            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            result.push(X86Instruction::single_op_instruction("setne", "al"));
            result.push(X86Instruction::single_op_instruction("push", "rax"));
            result.push(X86Instruction::single_op_instruction("jmp", label2));
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

fn get_new_label() -> &'static str {
    unsafe {
        LABEL_COUNT += 1;

        return Box::leak(format!(".L{}", LABEL_COUNT).into_boxed_str());
    }
}
