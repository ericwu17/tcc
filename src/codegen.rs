use crate::parser::BinOp;
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
    result.push_str("  pop rdi\n");
    result.push_str("  mov rax, 60\n");
    result.push_str("  syscall\n");

    result
}

fn generate_expr_code(expr: Expr) -> String {
    match expr {
        Expr::Int(v) => {
            return format!("  push  {}\n", v);
        }
        Expr::UnOp(op, inner_expr) => {
            let operation;
            match op {
                UnOp::Negation => {
                    operation = "  neg rdi\n";
                }
                UnOp::BitwiseComplement => {
                    operation = "  not rdi\n";
                }
                UnOp::Not => {
                    operation = "  cmp rdi, 0\n  mov rdi, 0\n  sete dil\n";
                }
            }

            let mut code = generate_expr_code(*inner_expr);
            code.push_str("  pop rdi\n");
            code.push_str(operation);
            code.push_str("  push rdi\n");
            return code;
        }
        Expr::BinOp(op, expr1, expr2) => {
            let expr_1_code = generate_expr_code(*expr1);
            let expr_2_code = generate_expr_code(*expr2);

            let mut code = String::new();
            code.push_str(&expr_1_code);
            code.push_str(&expr_2_code);

            code.push_str("  pop rsi\n"); // expr 2 in rsi
            code.push_str("  pop rdi\n"); // expr 1 in rdi

            match op {
                BinOp::Plus => code.push_str("  add rdi, rsi\n"),
                BinOp::Minus => code.push_str("  sub rdi, rsi\n"),
                BinOp::Multiply => code.push_str("  imul rdi, rsi\n"),
                BinOp::Divide => {
                    code.push_str("  mov eax, edi\n");
                    code.push_str("  cdq\n");
                    code.push_str("  idiv esi\n");
                    code.push_str("  mov rdi, rax\n");
                }
                _ => panic!(),
            }

            // final result goes into rdi
            code.push_str("  push rdi\n");

            return code;
        }
    }
}
