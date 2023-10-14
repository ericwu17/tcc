#![allow(dead_code)]

#[derive(Clone, Copy, Debug)]
pub enum Reg {
    Rsp,
    Rbp,
    Rax,
    Rdx,

    Rbx,
    Rcx,
    Rsi,
    Rdi,

    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

impl Reg {
    pub fn get_64_bit_name(&self) -> String {
        match self {
            Reg::Rsp => "rsp".to_owned(),
            Reg::Rbp => "rbp".to_owned(),
            Reg::Rax => "rax".to_owned(),
            Reg::Rdx => "rdx".to_owned(),
            Reg::Rbx => "rbx".to_owned(),
            Reg::Rcx => "rcx".to_owned(),
            Reg::Rsi => "rsi".to_owned(),
            Reg::Rdi => "rdi".to_owned(),
            Reg::R8 => "r8".to_owned(),
            Reg::R9 => "r9".to_owned(),
            Reg::R10 => "r10".to_owned(),
            Reg::R11 => "r11".to_owned(),
            Reg::R12 => "r12".to_owned(),
            Reg::R13 => "r13".to_owned(),
            Reg::R14 => "r14".to_owned(),
            Reg::R15 => "r15".to_owned(),
        }
    }

    pub fn get_32_bit_name(&self) -> String {
        match self {
            Reg::Rsp => "esp".to_owned(),
            Reg::Rbp => "ebp".to_owned(),
            Reg::Rax => "eax".to_owned(),
            Reg::Rdx => "edx".to_owned(),
            Reg::Rbx => "ebx".to_owned(),
            Reg::Rcx => "ecx".to_owned(),
            Reg::Rsi => "esi".to_owned(),
            Reg::Rdi => "edi".to_owned(),
            Reg::R8 => "r8d".to_owned(),
            Reg::R9 => "r9d".to_owned(),
            Reg::R10 => "r10d".to_owned(),
            Reg::R11 => "r11d".to_owned(),
            Reg::R12 => "r12d".to_owned(),
            Reg::R13 => "r13d".to_owned(),
            Reg::R14 => "r14d".to_owned(),
            Reg::R15 => "r15d".to_owned(),
        }
    }

    pub fn get_16_bit_name(&self) -> String {
        match self {
            Reg::Rsp => "sp".to_owned(),
            Reg::Rbp => "bp".to_owned(),
            Reg::Rax => "ax".to_owned(),
            Reg::Rdx => "dx".to_owned(),
            Reg::Rbx => "bx".to_owned(),
            Reg::Rcx => "cx".to_owned(),
            Reg::Rsi => "si".to_owned(),
            Reg::Rdi => "di".to_owned(),
            Reg::R8 => "r8w".to_owned(),
            Reg::R9 => "r9w".to_owned(),
            Reg::R10 => "r10w".to_owned(),
            Reg::R11 => "r11w".to_owned(),
            Reg::R12 => "r12w".to_owned(),
            Reg::R13 => "r13w".to_owned(),
            Reg::R14 => "r14w".to_owned(),
            Reg::R15 => "r15w".to_owned(),
        }
    }

    pub fn get_8_bit_name(&self) -> String {
        match self {
            Reg::Rsp => "spl".to_owned(),
            Reg::Rbp => "bpl".to_owned(),
            Reg::Rax => "al".to_owned(),
            Reg::Rdx => "dl".to_owned(),
            Reg::Rbx => "bl".to_owned(),
            Reg::Rcx => "cl".to_owned(),
            Reg::Rsi => "sil".to_owned(),
            Reg::Rdi => "dil".to_owned(),
            Reg::R8 => "r8b".to_owned(),
            Reg::R9 => "r9b".to_owned(),
            Reg::R10 => "r10b".to_owned(),
            Reg::R11 => "r11b".to_owned(),
            Reg::R12 => "r12b".to_owned(),
            Reg::R13 => "r13b".to_owned(),
            Reg::R14 => "r14b".to_owned(),
            Reg::R15 => "r15b".to_owned(),
        }
    }

    pub fn get_default_name(&self) -> String {
        match self {
            Reg::Rsp | Reg::Rbp => self.get_64_bit_name(),
            _ => self.get_32_bit_name(),
        }
    }
}
