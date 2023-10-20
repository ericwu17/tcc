use std::fmt;

use crate::parser::expr_parser::{BinOp, UnOp};

use super::{Identifier, TacVal};

pub enum TacInstr {
    Exit(TacVal),
    Return(TacVal),
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
            TacInstr::BinOp(ident, _, _, _) => {
                result = Some(*ident);
            }
            TacInstr::UnOp(ident, _, _) => {
                result = Some(*ident);
            }
            TacInstr::Copy(ident, _) => {
                result = Some(*ident);
            }
            TacInstr::Label(..)
            | TacInstr::Jmp(..)
            | TacInstr::Exit(..)
            | TacInstr::JmpNotZero(..)
            | TacInstr::JmpZero(..)
            | TacInstr::Return(_) => {}
            TacInstr::Call(_, _, optional_ident) => result = *optional_ident,
            TacInstr::LoadArg(ident, _) => {
                result = Some(*ident);
            }
        }
        result
    }

    pub fn get_read_identifiers(&self) -> Vec<Identifier> {
        let mut result = Vec::new();
        match self {
            TacInstr::Exit(v) => {
                if let TacVal::Var(ident) = v {
                    result.push(*ident);
                }
            }
            TacInstr::BinOp(_, v1, v2, _) => {
                if let TacVal::Var(ident) = v1 {
                    result.push(*ident);
                }
                if let TacVal::Var(ident) = v2 {
                    result.push(*ident);
                }
            }
            TacInstr::UnOp(_, v, _) => {
                if let TacVal::Var(ident) = v {
                    result.push(*ident);
                }
            }
            TacInstr::Copy(_, v) => {
                if let TacVal::Var(ident) = v {
                    result.push(*ident);
                }
            }
            TacInstr::Label(..) | TacInstr::Jmp(..) | TacInstr::LoadArg(_, _) => {}
            TacInstr::JmpNotZero(_, v) => {
                if let TacVal::Var(ident) = v {
                    result.push(*ident);
                }
            }
            TacInstr::JmpZero(_, v) => {
                if let TacVal::Var(ident) = v {
                    result.push(*ident);
                }
            }
            TacInstr::Call(_, args, _) => {
                for arg in args {
                    if let TacVal::Var(ident) = arg {
                        result.push(*ident);
                    }
                }
            }
            TacInstr::Return(v) => {
                if let TacVal::Var(ident) = v {
                    result.push(*ident);
                }
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
            TacInstr::Exit(v) => {
                write!(f, "exit {:?}", v)
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
        }
    }
}
