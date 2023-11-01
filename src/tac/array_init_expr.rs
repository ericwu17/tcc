use crate::{
    parser::expr_parser::{BinOp, Expr, ExprEnum},
    types::{VarSize, VarType},
};

use super::{
    expr::{generate_expr_tac, get_type_size, ValTarget},
    get_new_temp_name,
    tac_instr::TacInstr,
    CodeEnv, Identifier, TacVal,
};

pub fn gen_arr_init_expr_tac(
    arr_type: &VarType,
    arr_init_expr: &Expr,
    ptr_to_arr: Identifier,
    code_env: &CodeEnv,
) -> Vec<TacInstr> {
    // generates an array initializer expression by evaluating each expression (at runtime)
    // and storing each value into the array.
    let mut result = Vec::new();

    let exprs = match &arr_init_expr.content {
        ExprEnum::ArrInitExpr(x) => x,
        _ => unreachable!(),
    };

    let arr_type_size = arr_type.num_bytes() as i64;

    for expr in exprs {
        match &expr.content {
            ExprEnum::ArrInitExpr(_) => {
                let inner_type = match arr_type {
                    VarType::Arr(inner, _) => inner,
                    VarType::Fund(_) | VarType::Ptr(_) => unreachable!(),
                };

                let new_ptr = get_new_temp_name(VarSize::Quad);
                result.push(TacInstr::Copy(new_ptr, TacVal::Var(ptr_to_arr)));

                let instrs = gen_arr_init_expr_tac(inner_type, expr, new_ptr, code_env);
                result.extend(instrs);
            }
            _ => {
                let size = get_type_size(arr_type).unwrap();
                let val_tmp = get_new_temp_name(size);
                let (expr_instrs, tac_val) =
                    generate_expr_tac(expr, code_env, ValTarget::Ident(val_tmp));
                result.extend(expr_instrs);
                result.push(TacInstr::DerefStore(ptr_to_arr, tac_val));
            }
        }

        result.push(TacInstr::BinOp(
            ptr_to_arr,
            TacVal::Var(ptr_to_arr),
            TacVal::Lit(arr_type_size, VarSize::Quad),
            BinOp::Plus,
        ))
    }

    result
}

pub fn gen_opt_arr_init_expr_tac(
    arr_type: &VarType,
    num_elements: usize,
    arr_init_expr: &Expr,
    ptr_to_arr: Identifier,
) -> Option<Vec<TacInstr>> {
    if let Some(bytes) = gen_arr_init_expr_bytes(arr_type, num_elements, arr_init_expr) {
        let mut result = Vec::new();

        let element_size = arr_type.num_bytes();
        result.push(TacInstr::MemChunk(
            ptr_to_arr,
            num_elements * element_size,
            Some(bytes),
        ));

        Some(result)
    } else {
        None
    }
}

fn gen_arr_init_expr_bytes(
    arr_type: &VarType,
    num_elements: usize,
    arr_init_expr: &Expr,
) -> Option<Vec<u8>> {
    let exprs = match &arr_init_expr.content {
        ExprEnum::ArrInitExpr(x) => x,
        _ => unreachable!(),
    };

    let mut bytes = Vec::new();
    let element_size = arr_type.num_bytes();

    for expr in exprs {
        match expr.content {
            ExprEnum::Int(value) => match element_size {
                1 => bytes.extend((value as i8).to_le_bytes()),
                2 => bytes.extend((value as i16).to_le_bytes()),
                4 => bytes.extend((value as i32).to_le_bytes()),
                8 => bytes.extend((value as i64).to_le_bytes()),
                _ => return None,
            },
            ExprEnum::ArrInitExpr(_) => {
                let (inner_type, inner_num_elements) = match arr_type {
                    VarType::Arr(a, b) => (a, b),
                    _ => unreachable!(),
                };

                let inner_bytes = gen_arr_init_expr_bytes(inner_type, *inner_num_elements, expr)?;
                bytes.extend(inner_bytes);
            }
            _ => return None,
        }
    }
    while bytes.len() < num_elements * element_size {
        bytes.push(0);
    }
    Some(bytes)
}
