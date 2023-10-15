use crate::{
    codegen::{gen_load_val_code, reg::Reg},
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
    assert_eq!(function_name, "putchar");
    assert_eq!(args.len(), 1);
    assert!(optional_ident.is_some());

    let argument = args.get(0).unwrap();

    gen_load_val_code(result, argument, Reg::Rdi, reg_alloc);
    result.push(X86Instr::Call {
        name: function_name.clone(),
    });
}
