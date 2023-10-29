use std::fmt;

use crate::parser::expr_parser::{BinOp, UnOp};

use super::{Identifier, TacVal};

pub enum TacInstr {
    Return(TacVal),
    MemChunk(Identifier, usize), // sets the identifier to a pointer pointing to a usize number of bytes
    Deref(Identifier, Identifier), // a = *b
    Ref(Identifier, Identifier), // a = &b
    BinOp(Identifier, TacVal, TacVal, BinOp),
    UnOp(Identifier, TacVal, UnOp),
    Copy(Identifier, TacVal),
    Label(String),
    Jmp(String),
    JmpZero(String, TacVal),
    JmpNotZero(String, TacVal),
    Call(String, Vec<TacVal>, Option<Identifier>),
    LoadArg(Identifier, usize), //loads an argument of the function, emitted at the beginning of the function body
}

impl TacInstr {
    pub fn get_written_identifier(&self) -> Option<Identifier> {
        let mut result = None;
        match self {
            TacInstr::LoadArg(ident, _)
            | TacInstr::BinOp(ident, _, _, _)
            | TacInstr::UnOp(ident, _, _)
            | TacInstr::Copy(ident, _)
            | TacInstr::Deref(ident, _)
            | TacInstr::Ref(ident, _)
            | TacInstr::MemChunk(ident, _) => {
                result = Some(*ident);
            }
            TacInstr::Label(..)
            | TacInstr::Jmp(..)
            | TacInstr::JmpNotZero(..)
            | TacInstr::JmpZero(..)
            | TacInstr::Return(_) => {}
            TacInstr::Call(_, _, optional_ident) => result = *optional_ident,
        }
        result
    }

    pub fn get_read_identifiers(&self) -> Vec<Identifier> {
        let mut result = Vec::new();
        match self {
            TacInstr::BinOp(_, v1, v2, _) => {
                if let TacVal::Var(ident) = v1 {
                    result.push(*ident);
                }
                if let TacVal::Var(ident) = v2 {
                    result.push(*ident);
                }
            }
            TacInstr::UnOp(_, v, _)
            | TacInstr::Copy(_, v)
            | TacInstr::JmpNotZero(_, v)
            | TacInstr::JmpZero(_, v)
            | TacInstr::Return(v) => {
                if let TacVal::Var(ident) = v {
                    result.push(*ident);
                }
            }

            TacInstr::Label(..)
            | TacInstr::Jmp(..)
            | TacInstr::LoadArg(_, _)
            | TacInstr::MemChunk(_, _) => {}

            TacInstr::Call(_, args, _) => {
                for arg in args {
                    if let TacVal::Var(ident) = arg {
                        result.push(*ident);
                    }
                }
            }
            TacInstr::Ref(_, ident) | TacInstr::Deref(_, ident) => {
                result.push(*ident);
            }
        }
        result
    }
}

impl fmt::Debug for TacInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TacInstr::BinOp(identifier, val1, val2, op) => {
                write!(f, "{:?} = {:?} {:?} {:?}", identifier, val1, op, val2)
            }
            TacInstr::UnOp(identifier, val, op) => {
                write!(f, "{:?} = {:?} {:?}", identifier, op, val)
            }
            TacInstr::Copy(identifier, val) => {
                write!(f, "{:?} = {:?}", identifier, val)
            }
            TacInstr::Label(label_name) => {
                write!(f, "{}:", label_name)
            }
            TacInstr::Jmp(label) => {
                write!(f, "jmp {}", label)
            }
            TacInstr::JmpZero(label, v) => {
                write!(f, "jz {} {:?}", label, v)
            }
            TacInstr::JmpNotZero(label, v) => {
                write!(f, "jnz {} {:?}", label, v)
            }
            TacInstr::Call(name, args, optional_ident) => match optional_ident {
                None => write!(f, "call {}({:?})", name, args),
                Some(ident) => write!(f, "{:?} = call {}({:?})", ident, name, args),
            },
            TacInstr::Return(v) => {
                write!(f, "return {:?}", v)
            }
            TacInstr::LoadArg(ident, index) => {
                write!(f, "{:?} is argument {}", ident, index)
            }
            TacInstr::MemChunk(ident, size) => {
                write!(f, "{:?} = alloc({})", ident, size)
            }
            TacInstr::Deref(ident1, ident2) => {
                write!(f, "{:?} = *{:?}", ident1, ident2)
            }
            TacInstr::Ref(ident1, ident2) => {
                write!(f, "{:?} = &{:?}", ident1, ident2)
            }
        }
    }
}
