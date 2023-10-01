use crate::parser::Expr;
use crate::parser::Program;
use crate::parser::Statement;
use crate::parser::UnOp;

pub fn generate_code(program: Program) -> String {
    let mut result = String::new();
    result.push_str("global _start\n");

    result.push_str("_start:\n");

    assert!(program.function.name == "main");

    match program.function.statement {
        Statement::Return(expr) => {
            result.push_str(&generate_expr_code(expr));
        }
    }

    result.push_str("  mov rax, 60\n");
    result.push_str("  syscall\n");

    result
}

fn generate_expr_code(expr: Expr) -> String {
    match expr {
        Expr::Int(v) => {
            return format!("  mov rdi, {}\n", v);
        }
        Expr::UnOp(op, inner_expr) => {
            let outer_code;

            match op {
                UnOp::Negation => {
                    outer_code = "  neg rdi\n";
                }
                UnOp::BitwiseComplement => {
                    outer_code = "  not rdi\n";
                }
                UnOp::Not => {
                    outer_code = "  cmp rdi, 0\n  mov rdi, 0\n  sete dil\n";
                }
            }

            let mut inner_code = generate_expr_code(*inner_expr);
            inner_code.push_str(outer_code);
            return inner_code;
        }
    }
}
