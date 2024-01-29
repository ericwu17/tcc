use crate::tac::{tac_instr::TacTransitionInstr, TacGenerator};

pub fn generate_continue_stmt_code(generator: &mut TacGenerator) {
    match generator.curr_context.loop_label_begin {
        Some(bb_ident) => generator.set_curr_bb_out_instr(TacTransitionInstr::Jmp(bb_ident)),
        None => {
            panic!("continue statement used outside loop");
        }
    }
}
