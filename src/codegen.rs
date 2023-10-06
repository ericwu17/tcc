mod expr_codegen;

use crate::parser::expr_parser::Expr;
use crate::parser::Program;
use crate::parser::Statement;
use std::collections::HashMap;

use self::expr_codegen::generate_expr_code;

pub struct X86Routine {
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

    fn single_instruction(operation: &str, operands: Vec<&str>) -> Self {
        X86Routine {
            instructions: vec![X86Instruction {
                operation: operation.to_string(),
                operands: operands.iter().map(|&x| x.to_owned()).collect(),
            }],
        }
    }
}

struct X86Instruction {
    operation: String,
    operands: Vec<String>,
}

impl X86Instruction {
    fn to_asm_code(&self) -> String {
        let mut result = String::new();
        result.push_str(&self.operation);
        result.push(' ');
        for operand in &self.operands {
            result.push_str(operand);
            result.push_str(", ");
        }
        return result;
    }
    fn no_operands_instr(operation: &str) -> Self {
        X86Instruction {
            operation: operation.to_owned(),
            operands: vec![],
        }
    }
    fn single_op_instruction(operation: &str, operand: &str) -> Self {
        X86Instruction {
            operation: operation.to_owned(),
            operands: vec![operand.to_owned()],
        }
    }
    fn double_op_instruction(operation: &str, operand1: &str, operand2: &str) -> Self {
        X86Instruction {
            operation: operation.to_owned(),
            operands: vec![operand1.to_owned(), operand2.to_owned()],
        }
    }
}

pub fn generate_code(program: Program) -> String {
    let mut result = String::new();
    result.push_str("global _start\n");
    result.push_str("_start:\n");

    assert_eq!(program.function.name, "main");

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
            "sub",
            "rsp",
            &format!("{}", space_needed),
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
        result.push(X86Instruction::no_operands_instr("ret"));
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
) -> String {
    // go through most local scopes first
    for var_map in curr_variable_map_list.iter().rev() {
        if let Some(offset) = var_map.get(var_name) {
            return format!("{}", *offset * 8);
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
            result.push(X86Instruction::no_operands_instr("syscall"));
            return result;
        }
        Statement::Declare(var_name, opt_value) => {
            if let Statement::Declare(var_name, _) = statement {
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
                    let var_location =
                        format!("[rbp - {}]", resolve_variable(var_name, var_map_list));

                    let mut result = generate_expr_code(expr, var_map_list);
                    result.push(X86Instruction::single_op_instruction("pop", "rdi"));
                    result.push(X86Instruction::double_op_instruction(
                        "mov",
                        &var_location,
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
        Statement::If(condition, taken, not_taken) => {
            return generate_if_statement_code(
                condition,
                taken,
                not_taken.as_deref(),
                var_map_list,
                curr_var_index,
            );
        }
        Statement::While(condition, body) => {
            return generate_while_loop_code(condition, body, var_map_list, curr_var_index);
        }
        Statement::Break => {
            todo!();
        }
        Statement::Continue => {
            todo!();
        }
    }
}

fn get_new_label() -> String {
    unsafe {
        LABEL_COUNT += 1;

        return format!(".L{}", LABEL_COUNT);
    }
}

fn generate_if_statement_code(
    condition: &Expr,
    taken: &Statement,
    not_taken: Option<&Statement>,
    var_map_list: &mut Vec<HashMap<String, usize>>,
    curr_var_index: &mut usize,
) -> X86Routine {
    let label_not_taken = get_new_label();
    let label_end = get_new_label();

    let mut result = generate_expr_code(condition, var_map_list);
    result.push(X86Instruction::single_op_instruction("pop", "rdi"));
    result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
    result.push(X86Instruction::single_op_instruction(
        "je",
        &label_not_taken,
    ));

    let taken_routine = generate_statement_code(taken, var_map_list, curr_var_index);
    result.extend(taken_routine);
    if not_taken.is_some() {
        result.push(X86Instruction::single_op_instruction("jmp", &label_end));
    }
    result.push(X86Instruction::no_operands_instr(&label_not_taken));

    if not_taken.is_some() {
        let not_taken_routine =
            generate_statement_code(not_taken.unwrap(), var_map_list, curr_var_index);
        result.extend(not_taken_routine);
        result.push(X86Instruction::no_operands_instr(&label_end));
    }

    result
}

fn generate_while_loop_code(
    condition: &Expr,
    body: &Statement,
    var_map_list: &mut Vec<HashMap<String, usize>>,
    curr_var_index: &mut usize,
) -> X86Routine {
    let start_label = get_new_label();
    let end_label = get_new_label();

    let mut result = X86Routine::new();
    result.push(X86Instruction::no_operands_instr(&start_label));
    result.extend(generate_expr_code(condition, var_map_list));
    result.push(X86Instruction::single_op_instruction("pop", "rdi"));
    result.push(X86Instruction::double_op_instruction("cmp", "rdi", "0"));
    result.push(X86Instruction::single_op_instruction("je", &end_label));
    result.extend(generate_statement_code(body, var_map_list, curr_var_index));
    result.push(X86Instruction::single_op_instruction("jmp", &start_label));
    result.push(X86Instruction::no_operands_instr(&end_label));

    result
}
