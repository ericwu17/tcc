use crate::{
    parser::expr_parser::{BinOp, Expr, ExprEnum},
    tac::{get_type_size, tac_instr::TacBBInstr, Identifier, TacGenerator, TacVal},
    types::{VarSize, VarType},
};

pub fn generate_declaration_tac(
    generator: &mut TacGenerator,
    var_name: &String,
    opt_value: &Option<Expr>,
    t: &VarType,
) {
    match t {
        VarType::Fund(_) | VarType::Ptr(_) => {
            let map = generator.curr_context.get_var_map_mut();
            if map.get(var_name).is_some() {
                panic!(
                    "doubly declared variable (should have been caught by check_vars): {}",
                    var_name
                );
            }

            match opt_value {
                Some(expr) => {
                    let result = generator.consume_expr(expr, Some(get_type_size(t)));

                    let map = generator.curr_context.get_var_map_mut();
                    map.insert(var_name.clone(), result);
                }
                None => {
                    let var_temp_loc = generator.get_new_temp_name(get_type_size(t));
                    let map = generator.curr_context.get_var_map_mut();
                    map.insert(var_name.clone(), var_temp_loc);
                }
            }
        }
        VarType::Arr(inner_type, num_elements) => {
            let arr_ptr_identifier = generator.get_new_temp_name(VarSize::Quad);
            let map = generator.curr_context.get_var_map_mut();
            map.insert(var_name.clone(), arr_ptr_identifier);

            let num_bytes = inner_type.num_bytes() * num_elements;
            match opt_value {
                None => {
                    generator.push_instr(TacBBInstr::MemChunk(arr_ptr_identifier, num_bytes, None));
                }
                Some(arr_init_expr) => {
                    if let Some(bytes) =
                        gen_arr_init_expr_bytes(inner_type, *num_elements, arr_init_expr)
                    {
                        generator.push_instr(TacBBInstr::MemChunk(
                            arr_ptr_identifier,
                            num_bytes,
                            Some(bytes),
                        ))
                    } else {
                        let new_ptr = generator.get_new_temp_name(VarSize::Quad);
                        generator.push_instr(TacBBInstr::MemChunk(
                            arr_ptr_identifier,
                            num_bytes,
                            None,
                        ));
                        generator
                            .push_instr(TacBBInstr::Copy(new_ptr, TacVal::Var(arr_ptr_identifier)));

                        gen_arr_init_expr_tac(generator, inner_type, arr_init_expr, new_ptr);
                    }
                }
            }
        }
    }
}

/// generates an array initializer expression by evaluating each expression (at runtime)
/// and storing each value into the array.
pub fn gen_arr_init_expr_tac(
    generator: &mut TacGenerator,
    arr_type: &VarType,
    arr_init_expr: &Expr,
    ptr_to_arr: Identifier,
) {
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

                let new_ptr = generator.get_new_temp_name(VarSize::Quad);
                generator.push_instr(TacBBInstr::Copy(new_ptr, TacVal::Var(ptr_to_arr)));

                gen_arr_init_expr_tac(generator, inner_type, expr, new_ptr);
            }
            _ => {
                let size = get_type_size(arr_type);
                let expr_ident = generator.consume_expr(expr, Some(size));
                generator.push_instr(TacBBInstr::DerefStore(ptr_to_arr, TacVal::Var(expr_ident)));
            }
        }

        // increment the pointer to array for initialization
        generator.push_instr(TacBBInstr::BinOp(
            ptr_to_arr,
            TacVal::Var(ptr_to_arr),
            TacVal::Lit(arr_type_size, VarSize::Quad),
            BinOp::Plus,
        ))
    }
}

/// Returns None if the init expression cannot be
/// calculated during compilation.
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
                8 => bytes.extend(value.to_le_bytes()),
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
