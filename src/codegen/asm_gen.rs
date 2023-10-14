use super::{CCode, Location, Reg, X86Instr};

fn convert_reg_to_asm(reg: &Reg) -> String {
    // returns the 32-bit version of the corresponding register,
    // but the stack pointers rsp and rbp get full 64-bit names.
    match reg {
        Reg::Rsp => "rsp".to_owned(),
        Reg::Rbp => "rbp".to_owned(),
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

fn convert_reg_to_asm_64_bit(reg: &Reg) -> String {
    // returns the 32-bit version of the corresponding register
    match reg {
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

fn convert_location_to_asm(location: &Location) -> String {
    match location {
        Location::Mem(offset) => format!("[rbp - {}]", offset),
        Location::Reg(r) => convert_reg_to_asm(r),
    }
}

fn convert_cc_to_suffix(cc: &CCode) -> String {
    match cc {
        CCode::E => "e".to_owned(),
        CCode::NE => "ne".to_owned(),
        CCode::L => "l".to_owned(),
        CCode::LE => "le".to_owned(),
        CCode::G => "g".to_owned(),
        CCode::GE => "ge".to_owned(),
    }
}

pub fn convert_to_asm(instr: &X86Instr) -> String {
    match instr {
        X86Instr::Push { reg } => format!("push {}", convert_reg_to_asm_64_bit(reg)),
        X86Instr::Pop { reg } => format!("pop {}", convert_reg_to_asm_64_bit(reg)),
        X86Instr::Mov { dst, src } => format!(
            "mov {}, {}",
            convert_location_to_asm(dst),
            convert_location_to_asm(src),
        ),
        X86Instr::MovImm { dst, imm } => format!("mov {}, {}", convert_location_to_asm(dst), imm),
        X86Instr::Add { dst, src } => format!(
            "add {}, {}",
            convert_reg_to_asm(dst),
            convert_reg_to_asm(src),
        ),
        X86Instr::Sub { dst, src } => format!(
            "sub {}, {}",
            convert_reg_to_asm(dst),
            convert_reg_to_asm(src),
        ),
        X86Instr::IMul { dst, src } => format!(
            "imul {}, {}",
            convert_reg_to_asm(dst),
            convert_reg_to_asm(src),
        ),
        X86Instr::SubImm { dst, imm } => format!("sub {}, {}", convert_reg_to_asm(dst), imm),
        X86Instr::Cdq => "cdq".to_owned(),
        X86Instr::Idiv { src } => format!("idiv {}", convert_reg_to_asm(src)),
        X86Instr::Label { name } => format!(".{}:", name),
        X86Instr::Jmp { label } => format!("jmp {}", label),
        X86Instr::JmpCC { label, condition } => {
            format!("jmp{} {}", convert_cc_to_suffix(condition), label)
        }
        X86Instr::SetCC { dst, condition } => format!(
            "jmp{} {}",
            convert_cc_to_suffix(condition),
            convert_reg_to_asm(dst),
        ),

        X86Instr::Test { src } => format!(
            "test {} {}",
            convert_reg_to_asm(src),
            convert_reg_to_asm(src),
        ),
        X86Instr::Cmp { left, right } => format!(
            "cmp {}, {}",
            convert_reg_to_asm(left),
            convert_reg_to_asm(right)
        ),
        X86Instr::Not { dst } => format!("not {}", convert_reg_to_asm(dst),),
        X86Instr::Neg { dst } => format!("neg {}", convert_reg_to_asm(dst),),
        X86Instr::Syscall => "syscall".to_owned(),
    }
}

pub fn generate_program_asm(instrs: &Vec<X86Instr>) -> String {
    let mut result = String::new();

    let indent = "  ";

    result.push_str("global _start\n");
    result.push_str("_start:\n");

    for instr in instrs {
        let instr_string = convert_to_asm(instr);
        if !instr_string.starts_with(".") {
            // we assume only labels begin with ".", and labels should not be indented.s
            result.push_str(indent);
        }
        result.push_str(&instr_string);
        result.push('\n');
    }

    result
}
