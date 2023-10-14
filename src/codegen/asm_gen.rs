use super::{Location, X86Instr};

fn convert_location_to_asm(location: &Location) -> String {
    match location {
        Location::Mem(offset) => format!("[rbp - {}]", offset),
        Location::Reg(r) => r.get_default_name(),
    }
}

pub fn convert_to_asm(instr: &X86Instr) -> String {
    match instr {
        X86Instr::Push { reg } => format!("push {}", reg.get_64_bit_name()),
        X86Instr::Pop { reg } => format!("pop {}", reg.get_64_bit_name()),
        X86Instr::Mov { dst, src } => format!(
            "mov {}, {}",
            convert_location_to_asm(dst),
            convert_location_to_asm(src),
        ),
        X86Instr::MovImm { dst, imm } => format!("mov {}, {}", convert_location_to_asm(dst), imm),
        X86Instr::Add { dst, src } => {
            format!("add {}, {}", dst.get_default_name(), src.get_default_name())
        }
        X86Instr::Sub { dst, src } => {
            format!("sub {}, {}", dst.get_default_name(), src.get_default_name())
        }
        X86Instr::IMul { dst, src } => {
            format!(
                "imul {}, {}",
                dst.get_default_name(),
                src.get_default_name()
            )
        }
        X86Instr::SubImm { dst, imm } => format!("sub {}, {}", dst.get_default_name(), imm),
        X86Instr::Cdq => "cdq".to_owned(),
        X86Instr::Idiv { src } => format!("idiv {}", src.get_default_name()),
        X86Instr::Label { name } => format!(".{}:", name),
        X86Instr::Jmp { label } => format!("jmp .{}", label),
        X86Instr::JmpCC { label, condition } => {
            format!("j{} .{}", condition.to_suffix(), label)
        }
        X86Instr::SetCC { dst, condition } => {
            format!("set{} {}", condition.to_suffix(), dst.get_8_bit_name(),)
        }

        X86Instr::Test { src } => format!(
            "test {}, {}",
            src.get_default_name(),
            src.get_default_name(),
        ),
        X86Instr::Cmp { left, right } => format!(
            "cmp {}, {}",
            left.get_default_name(),
            right.get_default_name(),
        ),
        X86Instr::Not { dst } => format!("not {}", dst.get_default_name(),),
        X86Instr::Neg { dst } => format!("neg {}", dst.get_default_name(),),
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
