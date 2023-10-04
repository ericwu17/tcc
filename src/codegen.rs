use std::collections::HashMap;

use crate::parser::BinOp;
use crate::parser::Expr;
use crate::parser::Function;
use crate::parser::Program;
use crate::parser::Statement;
use crate::parser::UnOp;

struct X86Routine {
    instructions: Vec<X86Instruction>,
}

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
            result.push_str(indent);
            result.push_str(&instr.to_asm_code());
            result.push('\n');
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

    let routine = generate_function_code(program.function);
    result.push_str(&routine.to_asm_code());

    return result;
}

fn generate_function_code(func: Function) -> X86Routine {
    let mut result = X86Routine::new();
    result.push(X86Instruction::single_op_instruction("push", "rbp"));
    result.push(X86Instruction::double_op_instruction("mov", "rbp", "rsp")); // rbp now points to base of stack frame, and will remain pointing there for the rest of the function

    // first, we generate code to detect all variable declarations and increment rsp by the correct amount.
    let mut variables = Vec::new();
    for statement in &func.statements {
        if let Statement::Declare(var_name, _) = statement {
            variables.push(var_name);
        }
    }

    // every variable gets 8 bytes of space, so we allocate it here
    let space_needed = variables.len() * 8;
    result.push(X86Instruction::double_op_instruction(
        "add",
        "rsp",
        Box::leak(format!("{}", space_needed).into_boxed_str()),
    ));
    let mut variable_map: HashMap<String, &'static str> = HashMap::new();
    for (index, var_name) in variables.into_iter().enumerate() {
        let offset = Box::leak(format!("{}", (index + 1) * 8).into_boxed_str());
        variable_map.insert(var_name.clone(), offset);
    }

    for statement in &func.statements {
        result.extend(generate_statement_code(statement, &variable_map));
    }

    // let result = generate_statement_code(func.statement);
    result.push(X86Instruction::double_op_instruction("mov", "rsp", "rbp")); // restore rsp to what it was before this function was called
    result.push(X86Instruction::single_op_instruction("pop", "rbp")); // rbp now points to base of stack frame of outer function

    return result;
}

fn generate_statement_code(
    statement: &Statement,
    variable_map: &HashMap<String, &'static str>,
) -> X86Routine {
    match statement {
        Statement::Return(expr) => {
            let mut result = generate_expr_code(expr, variable_map);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction("mov", "rax", "60"));
            result.push(X86Instruction {
                operation: "syscall",
                operands: vec![],
            });
            return result;
        }
        Statement::Declare(var_name, opt_value) => {
            match opt_value {
                Some(expr) => {
                    let var_location = Box::leak(
                        format!("[rbp - {}]", variable_map.get(var_name).unwrap()).into_boxed_str(),
                    );

                    let mut result = generate_expr_code(expr, variable_map);
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
            let mut result = generate_expr_code(expr, variable_map);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            return result;
        }
    }
}

fn generate_expr_code(expr: &Expr, variable_map: &HashMap<String, &'static str>) -> X86Routine {
    match expr {
        Expr::Var(var_name) => {
            let var_location = Box::leak(
                format!("[rbp - {}]", variable_map.get(var_name).unwrap()).into_boxed_str(),
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
                format!("[rbp - {}]", variable_map.get(var_name).unwrap()).into_boxed_str(),
            );

            let mut result = generate_expr_code(expr, variable_map);
            result.push(X86Instruction::single_op_instruction("pop", "rdi"));
            result.push(X86Instruction::double_op_instruction(
                "mov",
                var_location,
                "rdi",
            ));
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

            let mut code = generate_expr_code(inner_expr, variable_map);
            code.push(X86Instruction::single_op_instruction("pop", "rdi"));
            code.extend(action);
            code.push(X86Instruction::single_op_instruction("push", "rdi"));
            return code;
        }
        Expr::BinOp(op, expr1, expr2) => {
            let expr_1_code = generate_expr_code(expr1, variable_map);
            let expr_2_code = generate_expr_code(expr2, variable_map);

            let mut code = X86Routine::new();
            code.extend(expr_1_code);
            code.extend(expr_2_code);

            code.extend(generate_binop_code(op));

            return code;
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
            // TODO: implement short-circuiting of logical or
            code.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            code.push(X86Instruction::double_op_instruction("mov", "eax", "0"));
            code.push(X86Instruction::single_op_instruction("setne", "al"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "0"));
            code.push(X86Instruction::double_op_instruction("or", "rdi", "rax"));

            code.push(X86Instruction::double_op_instruction("cmp", "rsi", "0"));
            code.push(X86Instruction::double_op_instruction("mov", "eax", "0"));
            code.push(X86Instruction::single_op_instruction("setne", "al"));
            code.push(X86Instruction::double_op_instruction("or", "rdi", "rax"));
        }
        BinOp::LogicalAnd => {
            // TODO: implement short-circuiting of logical and
            code.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
            code.push(X86Instruction::double_op_instruction("mov", "eax", "0"));
            code.push(X86Instruction::single_op_instruction("setne", "al"));
            code.push(X86Instruction::double_op_instruction("mov", "rdi", "1"));
            code.push(X86Instruction::double_op_instruction("and", "rdi", "rax"));

            code.push(X86Instruction::double_op_instruction("cmp", "rsi", "0"));
            code.push(X86Instruction::double_op_instruction("mov", "eax", "0"));
            code.push(X86Instruction::single_op_instruction("setne", "al"));
            code.push(X86Instruction::double_op_instruction("and", "rdi", "rax"));
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
