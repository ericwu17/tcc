use crate::{
    parser::expr_parser::UnOp,
    tac::{Identifier, TacVal, VarSize},
};

use super::{gen_load_val_code, CCode, Location, Reg, RegisterAllocator, X86Instr};

pub fn gen_unop_code(
    result: &mut Vec<X86Instr>,
    dst_ident: &Identifier,
    val: &TacVal,
    op: UnOp,
    reg_alloc: &RegisterAllocator,
) {
    let working_reg = Reg::Rsi;

    gen_load_val_code(result, val, working_reg, reg_alloc);

    match op {
        UnOp::Negation => result.push(X86Instr::Neg {
            dst: working_reg,
            size: val.get_size(),
        }),
        UnOp::BitwiseComplement => result.push(X86Instr::Not {
            dst: working_reg,
            size: val.get_size(),
        }),
        UnOp::Not => {
            result.push(X86Instr::Test {
                src: working_reg,
                size: val.get_size(),
            });
            result.push(X86Instr::MovImm {
                dst: Location::Reg(working_reg),
                imm: 0,
                size: VarSize::Quad,
            });
            result.push(X86Instr::SetCC {
                dst: working_reg,
                condition: CCode::E,
            });
        }
    }

    result.push(X86Instr::Mov {
        dst: reg_alloc.get_location(*dst_ident),
        src: Location::Reg(working_reg),
        size: dst_ident.get_size(),
    });
}
