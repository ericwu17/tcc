use crate::{
    parser::expr_parser::BinOp,
    tac::{Identifier, TacVal},
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
    match op {
        BinOp::Multiply => result.push(X86Instr::IMul {
            dst: val1_reg,
            src: val2_reg,
        }),
        BinOp::Divide => {
            result.push(X86Instr::Mov {
                dst: Location::Reg(Reg::Rax),
                src: Location::Reg(val1_reg),
            }); // move dividend into eax
            result.push(X86Instr::Cdq); // converts the 32 bit quantity in eax to a sign-extended 64 bit quantity.
            result.push(X86Instr::Idiv { src: val2_reg });
            result.push(X86Instr::Mov {
                dst: Location::Reg(val1_reg),
                src: Location::Reg(Reg::Rax),
            }); // move result into val1_reg
        }
        BinOp::Modulus => {
            result.push(X86Instr::Mov {
                dst: Location::Reg(Reg::Rax),
                src: Location::Reg(val1_reg),
            }); // move dividend into eax
            result.push(X86Instr::Cdq); // converts the 32 bit quantity in eax to a sign-extended 64 bit quantity.
            result.push(X86Instr::Idiv { src: val2_reg });
            result.push(X86Instr::Mov {
                dst: Location::Reg(val1_reg),
                src: Location::Reg(Reg::Rdx),
            }); // move result into val1_reg
        }
        BinOp::Plus => result.push(X86Instr::Add {
            dst: val1_reg,
            src: val2_reg,
        }),
        BinOp::Minus => result.push(X86Instr::Sub {
            dst: val1_reg,
            src: val2_reg,
        }),
        BinOp::GreaterThan => generate_cmp_code(result, val1_reg, val2_reg, CCode::G),
        BinOp::GreaterThanEq => generate_cmp_code(result, val1_reg, val2_reg, CCode::GE),
        BinOp::LessThan => generate_cmp_code(result, val1_reg, val2_reg, CCode::L),
        BinOp::LessThanEq => generate_cmp_code(result, val1_reg, val2_reg, CCode::LE),
        BinOp::Equals => generate_cmp_code(result, val1_reg, val2_reg, CCode::E),
        BinOp::NotEquals => generate_cmp_code(result, val1_reg, val2_reg, CCode::NE),
        BinOp::LogicalAnd | BinOp::LogicalOr => unreachable!(), // unreachable because short-circuiting code was generated in conversion to Tac phase.
    }

    result.push(X86Instr::Mov {
        dst: reg_alloc.get_location(*dst_ident),
        src: Location::Reg(val1_reg),
    });
}

fn generate_cmp_code(result: &mut Vec<X86Instr>, reg1: Reg, reg2: Reg, cc: CCode) {
    // makes comparison between reg1 and reg2, stores result in reg1.
    result.push(X86Instr::Cmp {
        left: reg2,
        right: reg1,
    });
    result.push(X86Instr::MovImm {
        dst: Location::Reg(reg1),
        imm: 0,
    });
    result.push(X86Instr::SetCC {
        dst: reg1,
        condition: cc,
    });
}
