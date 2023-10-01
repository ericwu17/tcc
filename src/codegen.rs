use crate::parser::Program;

pub fn generate_code(program: Program) -> String {
    let mut result = String::new();
    result.push_str("global _start\n");

    result.push_str("_start:\n");


    assert!(program.function.name == "main");
    let exit_code = program.function.statement.expr;

    result.push_str("  mov rax, 60\n");
    result.push_str(&format!("  mov rdi, {}\n", exit_code));
    result.push_str("  syscall\n");

    result
}