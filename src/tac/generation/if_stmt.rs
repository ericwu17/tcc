use crate::{
    parser::{expr_parser::Expr, Statement},
    tac::{
        tac_func::BBIdentifier,
        tac_instr::{TacBasicBlock, TacTransitionInstr},
        TacGenerator, TacVal,
    },
};

pub fn generate_if_statement_tac(
    generator: &mut TacGenerator,
    ctrl_expr: &Expr,
    taken: &Statement,
    opt_not_taken: Option<&Statement>,
) {
    let ctrl_expr_ident = generator.consume_expr(ctrl_expr, None);

    let next_id: BBIdentifier = generator.current_output.basic_blocks.len();
    let taken_bb_id = next_id;
    let not_taken_bb_id = next_id + 1;
    let exit_bb_id = next_id + 2;

    let taken_bb = TacBasicBlock::new(taken_bb_id);
    let not_taken_bb = TacBasicBlock::new(not_taken_bb_id);
    let exit_bb = TacBasicBlock::new(exit_bb_id);
    generator.current_output.basic_blocks.push(taken_bb);
    generator.current_output.basic_blocks.push(not_taken_bb);
    generator.current_output.basic_blocks.push(exit_bb);

    generator.set_curr_bb_out_instr(TacTransitionInstr::JmpNotZero {
        if_not_zero: taken_bb_id,
        if_zero: not_taken_bb_id,
        conditional_val: TacVal::Var(ctrl_expr_ident),
    });

    generator.curr_context.current_bb = taken_bb_id;
    generator.consume_statement(taken);
    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(exit_bb_id));

    generator.curr_context.current_bb = not_taken_bb_id;
    if let Some(not_taken) = opt_not_taken {
        generator.consume_statement(not_taken);
    }
    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(exit_bb_id));

    generator.curr_context.current_bb = exit_bb_id;
}
