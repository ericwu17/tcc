use std::collections::HashSet;

use crate::{parser::global_strings::generate_global_strings_asm, types::VarSize};

use super::{builtin_functions::BUILTIN_FUNCTIONS, Location, X86Instr};

fn convert_location_to_asm(location: &Location, size: VarSize) -> String {
    match location {
        Location::Mem(offset) => format!("[rbp - {}]", offset),
        Location::MemAbove(offset) => format!("[rbp + {}]", offset),
        Location::Reg(r) => r.get_sized_name(size),
        Location::MemPointed(r) => format!("[{}]", r.get_64_bit_name()),
    }
}

fn convert_to_asm(instr: &X86Instr) -> String {
    match instr {
        X86Instr::Push { reg } => format!("push {}", reg.get_64_bit_name()),
        X86Instr::Pop { reg } => format!("pop {}", reg.get_64_bit_name()),
        X86Instr::Mov { dst, src, size } => format!(
            "mov {}, {}",
            convert_location_to_asm(dst, *size),
            convert_location_to_asm(src, *size),
        ),
        X86Instr::MovImm { dst, imm, size } => {
            if let Location::Mem(_) = dst {
                if size == &VarSize::Quad {
                    let mut result = format!("mov rdi, {}\n  ", imm);
                    result.push_str(&format!(
                        "mov qword {}, rdi",
                        convert_location_to_asm(dst, *size),
                    ));

                    return result;
                }
            }
            let size_str = match size {
                VarSize::Byte => "byte",
                VarSize::Word => "word",
                VarSize::Dword => "dword",
                VarSize::Quad => "qword",
            };
            format!(
                "mov {} {}, {}",
                size_str,
                convert_location_to_asm(dst, *size),
                imm
            )
        }
        X86Instr::Add { dst, src, size } => {
            format!(
                "add {}, {}",
                dst.get_sized_name(*size),
                src.get_sized_name(*size)
            )
        }
        X86Instr::Sub { dst, src, size } => {
            format!(
                "sub {}, {}",
                dst.get_sized_name(*size),
                src.get_sized_name(*size)
            )
        }
        X86Instr::IMul { dst, src } => {
            format!(
                "imul {}, {}",
                dst.get_default_name(),
                src.get_default_name()
            )
        }
        X86Instr::SubImm { dst, imm, size } => {
            format!("sub {}, {}", dst.get_sized_name(*size), imm)
        }
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

        X86Instr::Test { src, size } => format!(
            "test {}, {}",
            src.get_sized_name(*size),
            src.get_sized_name(*size),
        ),
        X86Instr::Cmp { left, right, size } => format!(
            "cmp {}, {}",
            left.get_sized_name(*size),
            right.get_sized_name(*size),
        ),
        X86Instr::Not { dst, size } => format!("not {}", dst.get_sized_name(*size),),
        X86Instr::Neg { dst, size } => format!("neg {}", dst.get_sized_name(*size),),
        X86Instr::Call { name } => format!("call .{}", name),
        X86Instr::SignExtend { reg, size } => format!(
            "movsx {}, {}",
            reg.get_64_bit_name(),
            reg.get_sized_name(*size)
        ),
        X86Instr::Ret => "ret".to_owned(),
        X86Instr::StartLabel => "_start:".to_owned(),
        X86Instr::MovStaticLabel { reg, label_name } => {
            format!("mov {}, {}", reg.get_64_bit_name(), label_name)
        }
    }
}

/// Converts the internal X86 representation into an assembly file format
pub fn generate_program_asm(instrs: &Vec<X86Instr>) -> String {
    let mut result = String::new();

    result.push_str(&generate_global_strings_asm());

    let indent = "  ";

    result.push_str("global _start\n");

    let mut called_functions = HashSet::new();

    for instr in instrs {
        let instr_string = convert_to_asm(instr);
        if !instr_string.starts_with('.') && instr != &X86Instr::StartLabel {
            // we assume only labels begin with ".", and labels should not be indented.s
            result.push_str(indent);
        }
        if let X86Instr::Call { name } = instr {
            called_functions.insert(name);
        }
        result.push_str(&instr_string);
        result.push('\n');
    }

    for func_decl in BUILTIN_FUNCTIONS {
        if called_functions.contains(&func_decl.name.to_owned()) {
            result.push_str(func_decl.asm_code);
        }
    }

    result
}
