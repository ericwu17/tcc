use crate::{
    codegen::{gen_load_val_code, reg::Reg, Location},
    tac::{Identifier, TacVal},
};

use super::{RegisterAllocator, X86Instr};

pub fn generate_function_call_code(
    result: &mut Vec<X86Instr>,
    function_name: &String,
    args: &Vec<TacVal>,
    optional_ident: Option<Identifier>,
    reg_alloc: &RegisterAllocator,
) {
    assert!(optional_ident.is_some());
    let function_return_val_ident = optional_ident.unwrap();

    let argument = args.get(0).unwrap();

    gen_load_val_code(result, argument, Reg::Rdi, reg_alloc);

    for (index, arg) in args.iter().enumerate() {
        if index < 6 {
            let arg_reg = match index {
                0 => Reg::Rdi,
                1 => Reg::Rsi,
                2 => Reg::Rdx,
                3 => Reg::Rcx,
                4 => Reg::R8,
                5 => Reg::R9,
                _ => unreachable!(),
            };
            gen_load_val_code(result, arg, arg_reg, reg_alloc);
        } else {
            todo!()
        }
    }

    result.push(X86Instr::Call {
        name: function_name.clone(),
    });

    result.push(X86Instr::Mov {
        dst: reg_alloc.get_location(function_return_val_ident),
        src: Location::Reg(Reg::Rax),
        size: function_return_val_ident.get_size(),
    });
}
