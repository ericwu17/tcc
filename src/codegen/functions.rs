use crate::{
    codegen::{gen_load_val_code, reg::Reg, Location},
    tac::{Identifier, TacVal},
};

use super::{RegisterAllocator, X86Instr};

pub fn generate_function_call_code(
    result: &mut Vec<X86Instr>,
    function_name: &str,
    args: &[TacVal],
    optional_ident: Option<Identifier>,
    reg_alloc: &RegisterAllocator,
) {
    // we reverse the order of args in order to store things to memory first.
    // this way if the arguments past the 6th arg require using any of the 6 registers used
    // for the first 6 arguments, conflicts may be avoided.
    // also must ensure that last arguments get pushed on stack first
    for (index, arg) in args.iter().enumerate().rev() {
        if index < 6 {
            let arg_reg = get_nth_arg_reg(index).unwrap();
            gen_load_val_code(result, arg, arg_reg, reg_alloc);
        } else {
            gen_load_val_code(result, arg, Reg::Rdi, reg_alloc);
            result.push(X86Instr::Push { reg: Reg::Rdi });
        }
    }

    result.push(X86Instr::Call {
        name: function_name.to_owned(),
    });

    if let Some(function_return_val_ident) = optional_ident {
        result.push(X86Instr::Mov {
            dst: reg_alloc.get_location(function_return_val_ident),
            src: Location::Reg(Reg::Rax),
            size: function_return_val_ident.get_size(),
        });
    }
}

pub fn gen_load_arg_code(
    result: &mut Vec<X86Instr>,
    ident: &Identifier,
    arg_num: usize,
    reg_alloc: &RegisterAllocator,
) {
    if arg_num < 6 {
        let source_reg = get_nth_arg_reg(arg_num).unwrap();
        result.push(X86Instr::Mov {
            dst: reg_alloc.get_location(*ident),
            src: Location::Reg(source_reg),
            size: ident.get_size(),
        });
    } else {
        result.push(X86Instr::Mov {
            dst: Location::Reg(Reg::Rdi),
            src: Location::MemAbove((arg_num - 4) * 8),
            size: ident.get_size(),
        });
        result.push(X86Instr::Mov {
            dst: reg_alloc.get_location(*ident),
            src: Location::Reg(Reg::Rdi),
            size: ident.get_size(),
        });
    }
}

pub fn get_nth_arg_reg(n: usize) -> Option<Reg> {
    match n {
        0 => Some(Reg::Rdi),
        1 => Some(Reg::Rsi),
        2 => Some(Reg::Rdx),
        3 => Some(Reg::Rcx),
        4 => Some(Reg::R8),
        5 => Some(Reg::R9),
        _ => None,
    }
}
