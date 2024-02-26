use crate::tac::tac_func::TacFunc;

pub mod builtin_functions;

#[derive(Debug)]
pub struct RiscvFunc {
    name: String,
    // instructions
    // labels
}

/// generates the code in risc-v format
pub fn generate_rv_code(_ir: &[TacFunc]) -> Vec<RiscvFunc> {
    Vec::new()
}

pub fn generate_asm_string(_code: &[RiscvFunc]) -> String {
    ".global _start
helloworld:
    .ascii \"Hello World\\n\"

_start:
    addi a7, zero, 64
    addi a0, zero, 1
    la a1, helloworld
    addi a2, zero, 12
    ecall


    addi a7, zero, 93
    addi a0, zero, 0
    addi a1, zero, 2
    mul  a0, a0, a1
    ecall"
        .to_owned()
}
