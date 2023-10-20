use super::tac_instr::TacInstr;

#[derive(Debug)]
pub struct TacFunc {
    pub name: String,
    pub body: Vec<TacInstr>,
}
