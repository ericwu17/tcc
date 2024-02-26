use crate::{
    parser::{
        expr_parser::{Expr, ExprEnum},
        Statement,
    },
    tac::{
        tac_func::BBIdentifier,
        tac_instr::{TacBasicBlock, TacTransitionInstr},
        TacGenerator, TacVal,
    },
};

pub fn generate_for_loop_tac(
    generator: &mut TacGenerator,
    init_stmt: &Statement,
    ctrl_expr: Option<&Expr>,
    post_expr: Option<&Expr>,
    loop_body: &Statement,
) {
    // TODO: ensure that this code generates for loop code which
    // satisfies static-single-assignment rules!!!! (important if I want the compiler to handle for loops properly)

    // REPLACE LOOP EXPRESSIONS WITH DEFAULT VALUES
    let ctrl_expr = ctrl_expr.unwrap_or(&Expr {
        content: ExprEnum::Int(1),
        type_: None,
    });
    let post_expr = post_expr.unwrap_or(&Expr {
        content: ExprEnum::Int(1),
        type_: None,
    });

    // INITIALIZE SCOPE FOR LOOP HEADER
    generator.consume_statement(init_stmt);

    // INITIALIZE BASIC BLOCKS
    let next_id: BBIdentifier = generator.current_output.basic_blocks.len();
    let ctrl_expr_bb_id = next_id;
    let loop_body_bb_id = next_id + 1;
    let post_expr_bb_id = next_id + 2;
    let exit_bb_id = next_id + 3;
    let ctrl_expr_bb = TacBasicBlock::new(ctrl_expr_bb_id);
    let loop_body_bb = TacBasicBlock::new(loop_body_bb_id);
    let post_expr_bb = TacBasicBlock::new(post_expr_bb_id);
    let exit_bb = TacBasicBlock::new(exit_bb_id);
    generator.current_output.basic_blocks.push(ctrl_expr_bb);
    generator.current_output.basic_blocks.push(loop_body_bb);
    generator.current_output.basic_blocks.push(post_expr_bb);
    generator.current_output.basic_blocks.push(exit_bb);

    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(ctrl_expr_bb_id));

    // GENERATE CTRL EXPR
    generator.curr_context.current_bb = ctrl_expr_bb_id;
    let ctrl_ident = generator.consume_expr(ctrl_expr, None);
    generator.set_curr_bb_out_instr(TacTransitionInstr::JmpNotZero {
        if_not_zero: loop_body_bb_id,
        if_zero: exit_bb_id,
        conditional_val: TacVal::Var(ctrl_ident),
    });

    // GENERATE LOOP BODY
    generator.curr_context.current_bb = loop_body_bb_id;

    let outer_loop_label_end = generator.curr_context.loop_label_begin;
    let outer_loop_label_begin = generator.curr_context.loop_label_end;
    generator.curr_context.loop_label_begin = Some(ctrl_expr_bb_id);
    generator.curr_context.loop_label_end = Some(exit_bb_id);

    generator.consume_statement(loop_body);

    generator.curr_context.loop_label_begin = outer_loop_label_end;
    generator.curr_context.loop_label_end = outer_loop_label_begin;

    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(post_expr_bb_id));

    // GENERATE POST EXPR
    generator.curr_context.current_bb = post_expr_bb_id;
    generator.consume_expr(post_expr, None);
    generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(ctrl_expr_bb_id));

    // SET GENERATOR TO EXIT
    generator.curr_context.current_bb = exit_bb_id;
}
