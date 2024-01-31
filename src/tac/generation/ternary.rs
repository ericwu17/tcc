use crate::{
    parser::expr_parser::Expr,
    tac::{
        tac_func::BBIdentifier,
        tac_instr::{TacBBInstr, TacBasicBlock, TacTransitionInstr},
        Identifier, TacGenerator, TacVal,
    },
    types::VarSize,
};

pub fn generate_ternary_statement_tac(
    generator: &mut TacGenerator,
    ctrl_expr: &Expr,
    expr_true: &Expr,
    expr_false: &Expr,
    size: Option<VarSize>,
) -> Identifier {
    let ctrl_expr_ident = generator.consume_expr(ctrl_expr, None);

    let out_ident = generator.get_new_temp_name(size.unwrap_or_default());

    let next_id: BBIdentifier = generator.current_output.basic_blocks.len();
    let true_bb_id = next_id;
    let false_bb_id = next_id + 1;
    let exit_bb_id = next_id + 2;

    let true_bb = TacBasicBlock::new(true_bb_id);
    let false_bb = TacBasicBlock::new(false_bb_id);
    let exit_bb = TacBasicBlock::new(exit_bb_id);
    generator.current_output.basic_blocks.push(true_bb);
    generator.current_output.basic_blocks.push(false_bb);
    generator.current_output.basic_blocks.push(exit_bb);

    generator.set_curr_bb_out_instr(TacTransitionInstr::JmpNotZero {
        if_not_zero: true_bb_id,
        if_zero: false_bb_id,
        conditional_val: TacVal::Var(ctrl_expr_ident),
    });

    generator.curr_context.current_bb = true_bb_id;
    let ident_true = generator.consume_expr(expr_true, None);
    generator.push_instr(TacBBInstr::Copy(out_ident, TacVal::Var(ident_true)));
    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(exit_bb_id));

    generator.curr_context.current_bb = false_bb_id;
    let ident_false = generator.consume_expr(expr_false, None);
    generator.push_instr(TacBBInstr::Copy(out_ident, TacVal::Var(ident_false)));
    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(exit_bb_id));

    generator.curr_context.current_bb = exit_bb_id;

    out_ident
}
