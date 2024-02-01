use crate::{
    parser::expr_parser::{BinOp, Expr, ExprEnum},
    tac::{tac_instr::TacBBInstr, Identifier, TacGenerator, TacVal},
    types::{VarSize, VarType},
};

pub enum PrefixPostfixOp {
    PrefixInc,
    PrefixDec,
    PostfixInc,
    PostfixDec,
}

pub fn gen_prefix_postfix_inc_dec_tac(
    generator: &mut TacGenerator,
    expr: &Expr,
    op: PrefixPostfixOp,
) -> Identifier {
    let should_return_old_val = match op {
        PrefixPostfixOp::PrefixInc | PrefixPostfixOp::PrefixDec => false,
        PrefixPostfixOp::PostfixInc | PrefixPostfixOp::PostfixDec => true,
    };
    let binary_op = match op {
        PrefixPostfixOp::PrefixInc | PrefixPostfixOp::PostfixInc => BinOp::Plus,
        PrefixPostfixOp::PrefixDec | PrefixPostfixOp::PostfixDec => BinOp::Minus,
    };

    let expr_type = expr.type_.clone().unwrap();

    match &expr.content {
        ExprEnum::Var(var_name) => {
            let ident_to_update = generator
                .curr_context
                .resolve_variable_to_temp_name(var_name);

            let old_ident = generator.get_new_temp_name(expr_type.to_size().unwrap_or_default());
            generator.push_instr(TacBBInstr::Copy(old_ident, TacVal::Var(ident_to_update)));

            let updated_ident: Identifier =
                generate_update_code(generator, ident_to_update, expr_type, binary_op);
            generator.push_instr(TacBBInstr::Copy(
                ident_to_update,
                TacVal::Var(updated_ident),
            ));

            if should_return_old_val {
                old_ident
            } else {
                updated_ident
            }
        }
        _ => unreachable!(),
    }
}

fn generate_update_code(
    generator: &mut TacGenerator,
    ident_to_update: Identifier,
    type_: VarType,
    binary_op: BinOp,
) -> Identifier {
    let change_amt = match &type_ {
        VarType::Fund(_) => 1,
        VarType::Ptr(inner) | VarType::Arr(inner, _) => inner.num_bytes(),
    };

    let new_ident = generator.get_new_temp_name(type_.to_size().unwrap_or_default());

    generator.push_instr(TacBBInstr::BinOp(
        new_ident,
        TacVal::Var(ident_to_update),
        TacVal::Lit(change_amt as i64, VarSize::Quad),
        binary_op,
    ));

    new_ident
}
