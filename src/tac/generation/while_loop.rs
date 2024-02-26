use crate::{
    parser::{expr_parser::Expr, Statement},
    tac::{
        tac_func::BBIdentifier,
        tac_instr::{PhiInstr, TacBasicBlock, TacTransitionInstr},
        TacGenerator, TacVal,
    },
};

pub fn generate_while_loop_tac(
    generator: &mut TacGenerator,
    ctrl_expr: &Expr,
    loop_body: &Statement,
) {
    let next_id: BBIdentifier = generator.current_output.basic_blocks.len();
    let init_bb_id = next_id - 1;
    let ctrl_expr_bb_id = next_id;
    let loop_body_bb_id = next_id + 1;
    let exit_bb_id = next_id + 2;

    let ctrl_expr_bb = TacBasicBlock::new(ctrl_expr_bb_id);
    let loop_body_bb = TacBasicBlock::new(loop_body_bb_id);
    let exit_bb = TacBasicBlock::new(exit_bb_id);
    generator.current_output.basic_blocks.push(ctrl_expr_bb);
    generator.current_output.basic_blocks.push(loop_body_bb);
    generator.current_output.basic_blocks.push(exit_bb);

    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(ctrl_expr_bb_id));
    let init_var_map = generator.curr_context.get_var_map().clone();

    generator.curr_context.current_bb = loop_body_bb_id;

    let outer_loop_label_end = generator.curr_context.loop_label_begin;
    let outer_loop_label_begin = generator.curr_context.loop_label_end;
    generator.curr_context.loop_label_begin = Some(ctrl_expr_bb_id);
    generator.curr_context.loop_label_end = Some(exit_bb_id);

    generator.consume_statement(loop_body);

    generator.curr_context.loop_label_begin = outer_loop_label_end;
    generator.curr_context.loop_label_end = outer_loop_label_begin;

    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(ctrl_expr_bb_id));
    let loop_end_var_map = generator.curr_context.get_var_map().clone();

    // generate the phi instructions
    let mut new_var_map = loop_end_var_map;
    let mut phi_instrs = Vec::new();
    for (var_name, ident) in init_var_map {
        if let Some(ident_1) = new_var_map.get(&var_name) {
            if ident != *ident_1 {
                // need phi instruction here
                let new_ident = generator.get_new_temp_name(ident.get_size());
                phi_instrs.push(PhiInstr(
                    new_ident,
                    vec![(init_bb_id, ident), (loop_body_bb_id, *ident_1)],
                ));
                new_var_map.insert(var_name, new_ident);

                // also need to update loop body in order to ensure that the body now refers to the correct variables.
                generator
                    .get_bb_by_id(loop_body_bb_id)
                    .update_ident(ident, new_ident);
            } else {
                new_var_map.insert(var_name, ident);
            }
        }
    }

    generator.curr_context.current_bb = ctrl_expr_bb_id;
    generator.get_curr_bb().phi_instrs = phi_instrs;
    generator.curr_context.var_map = new_var_map;
    let ctrl_ident = generator.consume_expr(ctrl_expr, None);
    generator.set_curr_bb_out_instr(TacTransitionInstr::JmpNotZero {
        if_not_zero: loop_body_bb_id,
        if_zero: exit_bb_id,
        conditional_val: TacVal::Var(ctrl_ident),
    });

    generator.curr_context.current_bb = exit_bb_id;
}
