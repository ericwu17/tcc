use crate::{
    parser::expr_parser::{BinOp, Expr, ExprEnum},
    tac::{tac_instr::TacBBInstr, Identifier, TacGenerator, TacVal},
    types::{VarSize, VarType},
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

    let t1 = expr1.type_.clone().unwrap_or_default();
    let t2 = expr2.type_.clone().unwrap_or_default();

    let ident1 = generator.consume_expr(expr1, t1.to_size());
    let ident2 = generator.consume_expr(expr2, t2.to_size());

    if op == BinOp::Plus || op == BinOp::Minus {
        return generate_add_sub_binop_tac(generator, op, t1, t2, ident1, ident2, size);
    }

    let new_ident = generator.get_new_temp_name(size.unwrap_or(VarSize::Quad));

    generator.push_instr(TacBBInstr::BinOp(
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

            generator.push_instr(TacBBInstr::Copy(temp_name_of_assignee, TacVal::Var(ident)));

            temp_name_of_assignee
        }
        ExprEnum::Deref(inner) => {
            let lhs_ident = generator.consume_expr(inner, Some(VarSize::Quad));
            let rhs_ident = generator.consume_expr(rhs, None);

            generator.push_instr(TacBBInstr::DerefStore(lhs_ident, TacVal::Var(rhs_ident)));

            rhs_ident
        }
        _ => unreachable!(), // already checked that lhs must be a l_value
    }
}

fn generate_add_sub_binop_tac(
    generator: &mut TacGenerator,
    op: BinOp,
    t1: VarType,
    t2: VarType,
    ident1: Identifier,
    ident2: Identifier,
    size: Option<VarSize>,
) -> Identifier {
    match (&t1, &t2) {
        (VarType::Fund(_), VarType::Fund(_)) => {
            let final_temp_name = generator.get_new_temp_name(size.unwrap_or_default());
            generator.push_instr(TacBBInstr::BinOp(
                final_temp_name,
                TacVal::Var(ident1),
                TacVal::Var(ident2),
                op,
            ));
            final_temp_name
        }
        (VarType::Fund(_), VarType::Ptr(inner_type))
        | (VarType::Fund(_), VarType::Arr(inner_type, _))
        | (VarType::Ptr(inner_type), VarType::Fund(_))
        | (VarType::Arr(inner_type, _), VarType::Fund(_)) => {
            let (offset_ident, pointer_ident) = if let VarType::Fund(_) = t1 {
                (ident1, ident2)
            } else {
                (ident2, ident1)
            };

            let ptr_size = inner_type.num_bytes();
            let offset_ident_scaled = generator.get_new_temp_name(VarSize::Quad);
            let final_temp_name = generator.get_new_temp_name(VarSize::Quad);
            generator.push_instr(TacBBInstr::BinOp(
                offset_ident_scaled,
                TacVal::Var(offset_ident),
                TacVal::Lit(ptr_size as i64, VarSize::Quad),
                BinOp::Multiply,
            ));
            generator.push_instr(TacBBInstr::BinOp(
                final_temp_name,
                TacVal::Var(offset_ident_scaled),
                TacVal::Var(pointer_ident),
                op,
            ));
            final_temp_name
        }

        (VarType::Ptr(_), VarType::Ptr(_))
        | (VarType::Ptr(_), VarType::Arr(_, _))
        | (VarType::Arr(_, _), VarType::Ptr(_))
        | (VarType::Arr(_, _), VarType::Arr(_, _)) => unreachable!(),
    }
}
