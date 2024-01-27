use crate::types::VarType;
use std::fmt;

use super::{tac_instr::TacBasicBlock, Identifier};

pub type BBIdentifier = usize;

pub struct TacFunc {
    pub name: String,
    pub args: Vec<(Identifier, VarType)>,
    pub basic_blocks: Vec<TacBasicBlock>, // entry point into function will be element zero of this Vec
}

impl fmt::Debug for TacFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Function with name {:?}\n", self.name)?;
        write!(f, "arguments: {:?}\n", self.args)?;

        for (index, bb) in self.basic_blocks.iter().enumerate() {
            write!(f, "Basic block {:?}\n", index)?;
            write!(f, "{:?}\n", bb)?;
        }

        Ok(())
    }
}
