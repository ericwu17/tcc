pub mod asm_gen;
pub mod binop;
pub mod builtin_functions;
pub mod functions;
pub mod reg;
pub mod unop;
use std::collections::HashMap;

use crate::tac::{tac_func::TacFunc, tac_instr::TacInstr, Identifier, TacVal, VarSize};

use self::{
    binop::gen_binop_code,
    functions::{gen_load_arg_code, generate_function_call_code},
    reg::Reg,
    unop::gen_unop_code,
};

pub struct RegisterAllocator {
    map: HashMap<Identifier, Location>,
}

impl RegisterAllocator {
    fn new(tac_instrs: &Vec<TacInstr>) -> (Self, usize) {
        let mut set_of_temporaries: Vec<Identifier> = Vec::new();

        for instr in tac_instrs {
            for ident in instr.get_read_identifiers() {
                if !set_of_temporaries.contains(&ident) {
                    eprintln!(
                        "warning: found read from temporary {:?} with writing first.",
                        ident
                    );
                }
            }
            if let Some(ident) = instr.get_written_identifier() {
                set_of_temporaries.push(ident);
            }
        }

        let mut map = HashMap::new();

        let mut bytes_needed = 0;

        for identifier in &set_of_temporaries {
            bytes_needed += identifier.get_num_bytes();
            map.insert(*identifier, Location::Mem(bytes_needed));
        }

        (RegisterAllocator { map }, bytes_needed)
    }

    fn get_location(&self, temporary: Identifier) -> Location {
        return *self.map.get(&temporary).unwrap();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CCode {
    E,
    NE,
    L,
    LE,
    G,
    GE,
}

impl CCode {
    pub fn to_suffix(&self) -> String {
        match self {
            CCode::E => "e".to_owned(),
            CCode::NE => "ne".to_owned(),
            CCode::L => "l".to_owned(),
            CCode::LE => "le".to_owned(),
            CCode::G => "g".to_owned(),
            CCode::GE => "ge".to_owned(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum X86Instr {
    Push {
        reg: Reg,
    }, // always pushes the 64 bit regs
    Pop {
        reg: Reg,
    }, // always pops the 64 bit regs
    Mov {
        dst: Location,
        src: Location,
        size: VarSize,
    },
    MovImm {
        dst: Location,
        imm: i32,
        size: VarSize,
    },
    Add {
        dst: Reg,
        src: Reg,
        size: VarSize,
    },
    Sub {
        dst: Reg,
        src: Reg,
        size: VarSize,
    },
    IMul {
        dst: Reg,
        src: Reg,
    },
    SubImm {
        dst: Reg,
        imm: i32,
        size: VarSize,
    },
    Cdq, // convert double to quad, sign extends eax into edx:eax
    Idiv {
        src: Reg,
    }, // divides rax by src, quotient stored in rax
    Label {
        name: String,
    },
    Jmp {
        label: String,
    },
    JmpCC {
        label: String,
        condition: CCode,
    },
    SetCC {
        dst: Reg,
        condition: CCode,
    },
    Test {
        src: Reg,
        size: VarSize,
    }, // does "test src, src", setting condition flags.
    Cmp {
        left: Reg,
        right: Reg,
        size: VarSize,
    },
    Not {
        dst: Reg,
        size: VarSize,
    }, // bitwise complement
    Neg {
        dst: Reg,
        size: VarSize,
    }, // negate the number (additive inverse)
    Call {
        name: String,
    },
    SignExtend {
        reg: Reg,
        size: VarSize,
    },
    Syscall,
    Ret,
    StartLabel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    Reg(Reg),
    Mem(usize), // usize represents offset from rbp
    MemAbove(usize),
}

pub fn generate_x86_code(tac_funcs: &Vec<TacFunc>) -> Vec<X86Instr> {
    let mut result = Vec::new();

    for func in tac_funcs {
        generate_function_x86(&mut result, func);
    }

    result
}

fn generate_function_x86(result: &mut Vec<X86Instr>, function: &TacFunc) {
    let (reg_alloc, num_bytes_needed) = RegisterAllocator::new(&function.body);

    // FUNCTION PROLOGUE
    if function.name == "main" {
        result.push(X86Instr::StartLabel);
    } else {
        result.push(X86Instr::Label {
            name: function.name.clone(),
        });
    }
    result.push(X86Instr::Push { reg: Reg::Rbp });
    result.push(X86Instr::Mov {
        dst: Location::Reg(Reg::Rbp),
        src: Location::Reg(Reg::Rsp),
        size: VarSize::Quad,
    });
    result.push(X86Instr::SubImm {
        dst: Reg::Rsp,
        imm: num_bytes_needed as i32,
        size: VarSize::Quad,
    });

    for instr in &function.body {
        gen_x86_for_tac(result, instr, &reg_alloc);
    }
}

fn gen_x86_for_tac(result: &mut Vec<X86Instr>, instr: &TacInstr, reg_alloc: &RegisterAllocator) {
    match instr {
        TacInstr::Exit(val) => {
            generate_function_call_code(
                result,
                &"exit".to_owned(),
                &vec![val.clone()],
                None,
                reg_alloc,
            );
        }
        TacInstr::BinOp(dst_ident, val1, val2, op) => {
            gen_binop_code(result, dst_ident, val1, val2, *op, reg_alloc);
        }
        TacInstr::UnOp(dst_ident, val, op) => gen_unop_code(result, dst_ident, val, *op, reg_alloc),
        TacInstr::Copy(dst_ident, src_val) => {
            gen_load_val_code(result, src_val, Reg::Rdi, reg_alloc);
            result.push(X86Instr::Mov {
                dst: reg_alloc.get_location(*dst_ident),
                src: Location::Reg(Reg::Rdi),
                size: dst_ident.get_size(),
            });
        }
        TacInstr::Label(label_name) => result.push(X86Instr::Label {
            name: label_name.clone(),
        }),
        TacInstr::Jmp(label_name) => result.push(X86Instr::Jmp {
            label: label_name.clone(),
        }),
        TacInstr::JmpZero(label_name, val) => {
            gen_load_val_code(result, val, Reg::Rdi, reg_alloc);
            result.push(X86Instr::Test {
                src: Reg::Rdi,
                size: val.get_size(),
            });
            result.push(X86Instr::JmpCC {
                label: label_name.clone(),
                condition: CCode::E,
            })
        }
        TacInstr::JmpNotZero(label_name, val) => {
            gen_load_val_code(result, val, Reg::Rdi, reg_alloc);
            result.push(X86Instr::Test {
                src: Reg::Rdi,
                size: val.get_size(),
            });
            result.push(X86Instr::JmpCC {
                label: label_name.clone(),
                condition: CCode::NE,
            })
        }
        TacInstr::Call(function_name, args, optional_ident) => {
            generate_function_call_code(result, function_name, args, *optional_ident, reg_alloc)
        }
        TacInstr::Return(val) => {
            gen_load_val_code(result, val, Reg::Rax, reg_alloc);
            // FUNCTION EPILOGUE: generate this before each return statement in function
            result.push(X86Instr::Mov {
                dst: Location::Reg(Reg::Rsp),
                src: Location::Reg(Reg::Rbp),
                size: VarSize::Quad,
            });
            result.push(X86Instr::Pop { reg: Reg::Rbp });
            result.push(X86Instr::Ret);
        }
        TacInstr::LoadArg(ident, arg_num) => gen_load_arg_code(result, ident, *arg_num, reg_alloc),
    }
}

fn gen_load_val_code(
    result: &mut Vec<X86Instr>,
    val: &TacVal,
    reg: Reg,
    reg_alloc: &RegisterAllocator,
) {
    match val {
        TacVal::Lit(imm, size) => result.push(X86Instr::MovImm {
            dst: Location::Reg(reg),
            imm: *imm,
            size: *size,
        }),
        TacVal::Var(var_ident) => {
            let loc = reg_alloc.get_location(*var_ident);
            result.push(X86Instr::Mov {
                dst: Location::Reg(reg),
                src: loc,
                size: val.get_size(),
            });
            if val.get_size() != VarSize::Quad {
                result.push(X86Instr::SignExtend {
                    reg: reg,
                    size: val.get_size(),
                });
            }
        }
    }
}
