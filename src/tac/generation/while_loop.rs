use crate::{
    parser::{expr_parser::Expr, Statement},
    tac::{
        tac_func::BBIdentifier,
        tac_instr::{TacBasicBlock, TacTransitionInstr},
        TacGenerator, TacVal,
    },
};

pub fn generate_while_loop_tac(
    generator: &mut TacGenerator,
    ctrl_expr: &Expr,
    loop_body: &Statement,
) {
    let next_id: BBIdentifier = generator.current_output.basic_blocks.len();
    let ctrl_expr_bb_id = next_id;
    let loop_body_bb_id = next_id + 1;
    let exit_bb_id = next_id + 2;

    let ctrl_expr_bb = TacBasicBlock::new(ctrl_expr_bb_id);
    let loop_body_bb = TacBasicBlock::new(loop_body_bb_id);
    let exit_bb = TacBasicBlock::new(exit_bb_id);
    generator.current_output.basic_blocks.push(ctrl_expr_bb);
    generator.current_output.basic_blocks.push(loop_body_bb);
    generator.current_output.basic_blocks.push(exit_bb);

    let curr_bb = generator.get_curr_bb();
    curr_bb.out_instr = TacTransitionInstr::Jmp(ctrl_expr_bb_id);

    generator.curr_context.current_bb = ctrl_expr_bb_id;
    let ctrl_ident = generator.consume_expr(ctrl_expr, None);
    generator.get_curr_bb().out_instr = TacTransitionInstr::JmpNotZero {
        if_not_zero: loop_body_bb_id,
        if_zero: exit_bb_id,
        conditional_val: TacVal::Var(ctrl_ident),
    };

    generator.curr_context.current_bb = loop_body_bb_id;
    generator.consume_statement(loop_body);
    generator.get_curr_bb().out_instr = TacTransitionInstr::Jmp(ctrl_expr_bb_id);

    generator.curr_context.current_bb = exit_bb_id;
}
