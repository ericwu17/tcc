use std::fmt;

use crate::parser::expr_parser::{BinOp, UnOp};

use super::{tac_func::BBIdentifier, Identifier, TacVal};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TacTransitionInstr {
    Jmp(BBIdentifier),
    JmpNotZero {
        if_not_zero: BBIdentifier,
        if_zero: BBIdentifier,
        conditional_val: TacVal,
    },
    Return(TacVal),

    /// A Null TAC transition instruction is used during the process
    /// of constructing the TAC. After the TAC is fully constructed there should be none
    /// Null values remaining.
    Null,
}
#[derive(Clone)]
pub enum TacBBInstr {
    MemChunk(Identifier, usize, Option<Vec<u8>>), // sets the identifier to a pointer pointing to a usize number of bytes
    StaticStrPtr(Identifier, String), // set identifier to a static string pointing to data specified by the string.
    Deref(Identifier, Identifier),    // a = *b
    Ref(Identifier, Identifier),      // a = &b
    DerefStore(Identifier, TacVal),   // *a = b
    BinOp(Identifier, TacVal, TacVal, BinOp),
    UnOp(Identifier, TacVal, UnOp),
    Copy(Identifier, TacVal),
    Call(Identifier, String, Vec<TacVal>), // (return value identifier, function name, args)
}

#[derive(Debug, Clone)]
pub struct PhiInstr(
    pub Identifier,
    pub Vec<(BBIdentifier, Identifier)>, // The last basic block, and which identifier to "take" for the phi function
);

/// A `TacBasicBlock` represents a chain of instructions which will be executed in order,
/// uninterrupted by branches.
/// A basic block always ends with a branch or a return.
#[derive(Debug, Clone)]
pub struct TacBasicBlock {
    pub phi_instrs: Vec<PhiInstr>,
    pub instrs: Vec<TacBBInstr>,
    pub out_instr: TacTransitionInstr,
    /// id corresponds to the basic block's index in the function array.
    pub id: BBIdentifier,
}

impl TacBasicBlock {
    pub fn new(id: BBIdentifier) -> Self {
        // When initializing a basic block, we will temporarily make the out_instr Null.
        // This should be changed by whoever is building upon this basic block.
        TacBasicBlock {
            phi_instrs: Vec::new(),
            instrs: Vec::new(),
            out_instr: TacTransitionInstr::Null,
            id,
        }
    }
}

impl TacBBInstr {
    pub fn get_written_identifier(&self) -> Option<Identifier> {
        
        match self {
            TacBBInstr::BinOp(ident, _, _, _)
            | TacBBInstr::UnOp(ident, _, _)
            | TacBBInstr::Copy(ident, _)
            | TacBBInstr::Deref(ident, _)
            | TacBBInstr::Ref(ident, _)
            | TacBBInstr::MemChunk(ident, _, _)
            | TacBBInstr::StaticStrPtr(ident, _)
            | TacBBInstr::Call(ident, _, _) => Some(*ident),
            TacBBInstr::DerefStore(_, _) => None,
        }
    }

    pub fn get_read_identifiers(&self) -> Vec<Identifier> {
        let mut result = Vec::new();
        match self {
            TacBBInstr::BinOp(_, v1, v2, _) => {
                if let TacVal::Var(ident) = v1 {
                    result.push(*ident);
                }
                if let TacVal::Var(ident) = v2 {
                    result.push(*ident);
                }
            }
            TacBBInstr::UnOp(_, v, _) | TacBBInstr::Copy(_, v) | TacBBInstr::DerefStore(_, v) => {
                if let TacVal::Var(ident) = v {
                    result.push(*ident);
                }
            }

            TacBBInstr::MemChunk(_, _, _)
            | TacBBInstr::Ref(_, _)
            | TacBBInstr::StaticStrPtr(_, _) => {}

            TacBBInstr::Call(_, _, args) => {
                for arg in args {
                    if let TacVal::Var(ident) = arg {
                        result.push(*ident);
                    }
                }
            }
            TacBBInstr::Deref(_, ident) => {
                result.push(*ident);
            }
        }
        result
    }
}

impl fmt::Debug for TacBBInstr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TacBBInstr::BinOp(identifier, val1, val2, op) => {
                write!(f, "{:?} = {:?} {:?} {:?}", identifier, val1, op, val2)
            }
            TacBBInstr::UnOp(identifier, val, op) => {
                write!(f, "{:?} = {:?} {:?}", identifier, op, val)
            }
            TacBBInstr::Copy(identifier, val) => {
                write!(f, "{:?} = {:?}", identifier, val)
            }
            TacBBInstr::Call(ident, name, args) => {
                write!(f, "{:?} = call {}({:?})", ident, name, args)
            }
            TacBBInstr::MemChunk(ident, size, _) => {
                write!(f, "{:?} = alloc({})", ident, size)
            }
            TacBBInstr::Deref(ident1, ident2) => {
                write!(f, "{:?} = *{:?}", ident1, ident2)
            }
            TacBBInstr::Ref(ident1, ident2) => {
                write!(f, "{:?} = &{:?}", ident1, ident2)
            }
            TacBBInstr::DerefStore(ident, v) => {
                write!(f, "*{:?} = {:?}", ident, v)
            }
            TacBBInstr::StaticStrPtr(ident, data) => {
                write!(f, "{:?} points to static string `{}`", ident, data)
            }
        }
    }
}
