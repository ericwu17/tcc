use crate::{
    parser::expr_parser::{BinOp, Expr, ExprEnum},
    tac::{tac_instr::TacBBInstr, Identifier, TacGenerator, TacVal},
    types::VarSize,
};

pub fn generate_binop_tac(
    generator: &mut TacGenerator,
    op: BinOp,
    expr1: &Expr,
    expr2: &Expr,
    size: Option<VarSize>,
) -> Identifier {
    if op == BinOp::LogicalAnd || op == BinOp::LogicalOr {
        todo!()
    }

    if op == BinOp::Assign {
        return generate_assignment_tac(generator, expr1, expr2);
    }

    let ident1 = generator.consume_expr(expr1, size);
    let ident2 = generator.consume_expr(expr2, size);

    let new_ident = generator.get_new_temp_name(size.unwrap_or(VarSize::Quad));

    // TODO: handle addition and subtraction of pointers

    let curr_bb_index = generator.curr_context.current_bb;
    let curr_bb = &mut generator.current_output.basic_blocks[curr_bb_index];
    curr_bb.instrs.push(TacBBInstr::BinOp(
        new_ident,
        TacVal::Var(ident1),
        TacVal::Var(ident2),
        op,
    ));

    new_ident
}

fn generate_assignment_tac(generator: &mut TacGenerator, lhs: &Expr, rhs: &Expr) -> Identifier {
    match &lhs.content {
        ExprEnum::Var(var_name) => {
            let temp_name_of_assignee = generator
                .curr_context
                .resolve_variable_to_temp_name(var_name);

            let ident = generator.consume_expr(rhs, Some(temp_name_of_assignee.get_size()));

            let curr_bb = generator.get_curr_bb();

            curr_bb
                .instrs
                .push(TacBBInstr::Copy(temp_name_of_assignee, TacVal::Var(ident)));

            temp_name_of_assignee
        }
        ExprEnum::Deref(inner) => {
            todo!()
            // let ident = generator.consume_expr(inner, Some(VarSize::Quad));
            // let rhs_ident = generator.consume_expr(rhs, None);

            // let curr_bb = generator.get_curr_bb();
            // curr_bb
            //     .instrs
            //     .push(TacBBInstr::DerefStore(ident, TacVal::Var(rhs_ident)));

            // rhs_ident
        }
        _ => unreachable!(), // already checked that lhs must be a l_value
    }
}
