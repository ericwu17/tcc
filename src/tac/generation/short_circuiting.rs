use crate::{
    parser::expr_parser::{BinOp, Expr},
    tac::{
        tac_func::BBIdentifier,
        tac_instr::{TacBBInstr, TacBasicBlock, TacTransitionInstr},
        Identifier, TacGenerator, TacVal,
    },
};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ShortCircuitingBinop {
    And,
    Or,
}

impl From<BinOp> for ShortCircuitingBinop {
    fn from(value: BinOp) -> Self {
        match value {
            BinOp::LogicalAnd => Self::And,
            BinOp::LogicalOr => Self::Or,
            _ => panic!("Invalid conversion from binop to short-circuiting binop"),
        }
    }
}

pub fn generate_short_circuiting_tac(
    generator: &mut TacGenerator,
    op: ShortCircuitingBinop,
    expr1: &Expr,
    expr2: &Expr,
) -> Identifier {
    let out_ident = generator.consume_expr(expr1, None);

    let next_id: BBIdentifier = generator.current_output.basic_blocks.len();
    let eval_rhs_bb_id = next_id;
    let exit_bb_id = next_id + 1;
    let eval_rhs_bb = TacBasicBlock::new(eval_rhs_bb_id);
    let exit_bb: crate::tac::tac_instr::TacBasicBlock = TacBasicBlock::new(exit_bb_id);
    generator.current_output.basic_blocks.push(eval_rhs_bb);
    generator.current_output.basic_blocks.push(exit_bb);

    let out_instr = match op {
        ShortCircuitingBinop::And => TacTransitionInstr::JmpNotZero {
            if_not_zero: eval_rhs_bb_id,
            if_zero: exit_bb_id,
            conditional_val: TacVal::Var(out_ident),
        },
        ShortCircuitingBinop::Or => TacTransitionInstr::JmpNotZero {
            if_not_zero: exit_bb_id,
            if_zero: eval_rhs_bb_id,
            conditional_val: TacVal::Var(out_ident),
        },
    };
    generator.set_curr_bb_out_instr(out_instr);

    // GENERATE BASIC BLOCK TO EVAL RHS
    generator.curr_context.current_bb = eval_rhs_bb_id;
    let expr_2_ident = generator.consume_expr(expr2, None);
    generator.push_instr(TacBBInstr::Copy(out_ident, TacVal::Var(expr_2_ident)));
    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(exit_bb_id));

    // END
    generator.curr_context.current_bb = exit_bb_id;

    out_ident
}
