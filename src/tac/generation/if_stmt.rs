use crate::{
    parser::{expr_parser::Expr, Statement},
    tac::{
        tac_func::BBIdentifier,
        tac_instr::{PhiInstr, TacBasicBlock, TacTransitionInstr},
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
    let init_var_map = generator.curr_context.get_var_map().clone();

    generator.curr_context.current_bb = taken_bb_id;
    generator.consume_statement(taken);
    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(exit_bb_id));
    let taken_var_map = generator.curr_context.get_var_map().clone();

    generator.curr_context.restore_var_map(init_var_map);
    generator.curr_context.current_bb = not_taken_bb_id;
    if let Some(not_taken) = opt_not_taken {
        generator.consume_statement(not_taken);
    }
    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(exit_bb_id));
    let not_taken_var_map = generator.curr_context.get_var_map().clone();

    // generate the phi instructions
    let mut new_var_map = taken_var_map;
    let mut phi_instrs = Vec::new();
    for (var_name, ident) in not_taken_var_map {
        if let Some(ident_1) = new_var_map.get(&var_name) {
            if ident != *ident_1 {
                // need phi instruction here
                let new_ident = generator.get_new_temp_name(ident.get_size());
                phi_instrs.push(PhiInstr(
                    new_ident,
                    vec![(not_taken_bb_id, ident), (taken_bb_id, *ident_1)],
                ));
                new_var_map.insert(var_name, new_ident);
            }
        } else {
            new_var_map.insert(var_name, ident);
        }
    }

    generator.curr_context.current_bb = exit_bb_id;
    generator.get_curr_bb().phi_instrs = phi_instrs;
    generator.curr_context.var_map = new_var_map;
}
