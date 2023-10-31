use crate::{
    parser::expr_parser::BinOp,
    tac::{expr::get_bigger_size, Identifier, TacVal},
    types::VarSize,
};

use super::{gen_load_val_code, CCode, Location, Reg, RegisterAllocator, X86Instr};

pub fn gen_binop_code(
    result: &mut Vec<X86Instr>,
    dst_ident: &Identifier,
    val1: &TacVal,
    val2: &TacVal,
    op: BinOp,
    reg_alloc: &RegisterAllocator,
) {
    let val1_reg = Reg::Rdi;
    let val2_reg = Reg::Rsi;

    gen_load_val_code(result, val1, val1_reg, reg_alloc);
    gen_load_val_code(result, val2, val2_reg, reg_alloc);

    let bigger_operand_size =
        get_bigger_size(Some(val1.get_size()), Some(val2.get_size())).unwrap();

    match op {
        BinOp::Multiply => result.push(X86Instr::IMul {
            dst: val1_reg,
            src: val2_reg,
        }),
        BinOp::Divide => {
            result.push(X86Instr::Mov {
                dst: Location::Reg(Reg::Rax),
                src: Location::Reg(val1_reg),
                size: val1.get_size(),
            }); // move dividend into eax
            result.push(X86Instr::Cdq); // converts the 32 bit quantity in eax to a sign-extended 64 bit quantity.
            result.push(X86Instr::Idiv { src: val2_reg });
            // after idiv instruction, result is stored in eax,
            // so it needs to be sign extended to rax.
            result.push(X86Instr::SignExtend {
                reg: Reg::Rax,
                size: VarSize::Dword,
            });
            result.push(X86Instr::Mov {
                dst: Location::Reg(val1_reg),
                src: Location::Reg(Reg::Rax),
                size: dst_ident.get_size(),
            }); // move result into val1_reg
        }
        BinOp::Modulus => {
            result.push(X86Instr::Mov {
                dst: Location::Reg(Reg::Rax),
                src: Location::Reg(val1_reg),
                size: val1.get_size(),
            }); // move dividend into eax
            result.push(X86Instr::Cdq); // converts the 32 bit quantity in eax to a sign-extended 64 bit quantity.
            result.push(X86Instr::Idiv { src: val2_reg });
            result.push(X86Instr::Mov {
                dst: Location::Reg(val1_reg),
                src: Location::Reg(Reg::Rdx),
                size: dst_ident.get_size(),
            }); // move result into val1_reg
        }
        BinOp::Plus => result.push(X86Instr::Add {
            dst: val1_reg,
            src: val2_reg,
            size: dst_ident.get_size(),
        }),
        BinOp::Minus => result.push(X86Instr::Sub {
            dst: val1_reg,
            src: val2_reg,
            size: dst_ident.get_size(),
        }),
        BinOp::GreaterThan => {
            generate_cmp_code(result, val1_reg, val2_reg, CCode::G, bigger_operand_size)
        }
        BinOp::GreaterThanEq => {
            generate_cmp_code(result, val1_reg, val2_reg, CCode::GE, bigger_operand_size)
        }
        BinOp::LessThan => {
            generate_cmp_code(result, val1_reg, val2_reg, CCode::L, bigger_operand_size)
        }
        BinOp::LessThanEq => {
            generate_cmp_code(result, val1_reg, val2_reg, CCode::LE, bigger_operand_size)
        }
        BinOp::Equals => {
            generate_cmp_code(result, val1_reg, val2_reg, CCode::E, bigger_operand_size)
        }
        BinOp::NotEquals => {
            generate_cmp_code(result, val1_reg, val2_reg, CCode::NE, bigger_operand_size)
        }
        BinOp::LogicalAnd | BinOp::LogicalOr => unreachable!(), // unreachable because short-circuiting code was generated in conversion to Tac phase.
        BinOp::Assign => unreachable!(), // unreachable because we will never generate TAC with the assign operator
    }

    result.push(X86Instr::Mov {
        dst: reg_alloc.get_location(*dst_ident),
        src: Location::Reg(val1_reg),
        size: dst_ident.get_size(),
    });
}

fn generate_cmp_code(result: &mut Vec<X86Instr>, reg1: Reg, reg2: Reg, cc: CCode, size: VarSize) {
    // makes comparison between reg1 and reg2, stores result in reg1.
    result.push(X86Instr::Cmp {
        left: reg1,
        right: reg2,
        size,
    });
    result.push(X86Instr::MovImm {
        dst: Location::Reg(reg1),
        imm: 0,
        size: VarSize::Quad,
    });
    result.push(X86Instr::SetCC {
        dst: reg1,
        condition: cc,
    });
}
