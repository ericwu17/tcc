use crate::types::VarType;

use super::{tac_instr::TacInstr, Identifier};

#[derive(Debug)]
pub struct TacFunc {
    pub name: String,
    pub args: Vec<(Identifier, VarType)>,
    pub body: Vec<TacInstr>,
}
