use crate::{
    parser::expr_parser::Expr,
    tac::{get_expr_size, get_type_size, tac_instr::TacBBInstr, Identifier, TacGenerator},
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
            let var_map_list = &mut generator.curr_context.var_map_list;
            let last_elem_index = var_map_list.len() - 1;
            let this_scopes_variable_map = var_map_list.get_mut(last_elem_index).unwrap();
            if this_scopes_variable_map.get(var_name).is_some() {
                panic!(
                    "doubly declared variable (should have been caught by check_vars): {}",
                    var_name
                );
            }

            match opt_value {
                Some(expr) => {
                    let result = generator.consume_expr(expr, Some(get_type_size(t)));

                    let var_map_list = &mut generator.curr_context.var_map_list;
                    let last_elem_index = var_map_list.len() - 1;
                    let this_scopes_variable_map: &mut std::collections::HashMap<
                        String,
                        Identifier,
                    > = var_map_list.get_mut(last_elem_index).unwrap();
                    this_scopes_variable_map.insert(var_name.clone(), result);
                }
                None => {
                    let var_temp_loc = generator.get_new_temp_name(get_type_size(t));
                    let var_map_list = &mut generator.curr_context.var_map_list;
                    let last_elem_index = var_map_list.len() - 1;
                    let this_scopes_variable_map = var_map_list.get_mut(last_elem_index).unwrap();
                    this_scopes_variable_map.insert(var_name.clone(), var_temp_loc);
                }
            }
        }
        VarType::Arr(inner_type, num_elements) => {
            todo!();
            // let var_map_list = &mut generator.curr_context.var_map_list;
            // let last_elem_index = var_map_list.len() - 1;
            // let this_scopes_variable_map = var_map_list.get_mut(last_elem_index).unwrap();
            // let arr_ptr_identifier = generator.get_new_temp_name(VarSize::Quad);
            // this_scopes_variable_map.insert(var_name.clone(), arr_ptr_identifier);

            // let num_bytes = inner_type.num_bytes() * num_elements;

            // let mut result = Vec::new();
            // result.push(TacBBInstr::MemChunk(arr_ptr_identifier, num_bytes, None));

            // if let Some(arr_init_expr) = opt_value {
            //     if let Some(res) = gen_opt_arr_init_expr_tac(
            //         inner_type,
            //         *num_elements,
            //         arr_init_expr,
            //         arr_ptr_identifier,
            //     ) {
            //         return res;
            //     }
            //     let ptr_to_arr = get_new_temp_name(VarSize::Quad);
            //     result.push(TacInstr::Copy(ptr_to_arr, TacVal::Var(arr_ptr_identifier)));
            //     result.extend(gen_arr_init_expr_tac(
            //         inner_type,
            //         arr_init_expr,
            //         ptr_to_arr,
            //         code_env,
            //     ));
            // }
            // result
        }
    }
}
