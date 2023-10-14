use std::collections::HashMap;
use std::fmt;
pub mod expr;
pub mod loops;
pub mod prefix_postfix_inc_dec;
pub mod tac_instr;

use crate::parser::{expr_parser::Expr, Program, Statement};

use self::{
    expr::generate_expr_tac,
    loops::{gen_for_loop_tac, gen_while_loop_tac, generate_break_tac, generate_continue_tac},
    tac_instr::TacInstr,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Identifier(usize); // an identifier for a temporary in TAC

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

pub enum TacVal {
    Lit(i32),
    Var(Identifier),
}

impl fmt::Debug for TacVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TacVal::Lit(v) => write!(f, "{}", v),
            TacVal::Var(ident) => write!(f, "{:?}", ident),
        }
        // write!(f, "Point [{} {}]", self.x, self.y)
    }
}

pub struct CodeEnv {
    // a list of maps, one for each scope level, mapping variable names to temporary storage names
    pub var_map_list: Vec<HashMap<String, Identifier>>,

    // The two loop labels are used for break and continue statement code
    pub loop_label_end: Option<String>,
    pub loop_label_begin: Option<String>,
}

impl CodeEnv {
    fn new() -> Self {
        CodeEnv {
            var_map_list: Vec::new(),
            loop_label_end: None,
            loop_label_begin: None,
        }
    }
}

static mut TEMP_STORAGE_NUMBER: usize = 0;
static mut LABEL_NUMBER: usize = 0;

fn get_new_temp_name() -> Identifier {
    unsafe {
        // Safety: no race conditions because this compiler is single-threaded
        TEMP_STORAGE_NUMBER += 1;
        return Identifier(TEMP_STORAGE_NUMBER - 1);
    }
}

fn get_new_label_number() -> usize {
    unsafe {
        // Safety: no race conditions because this compiler is single-threaded
        LABEL_NUMBER += 1;
        return LABEL_NUMBER - 1;
    }
}

pub fn generate_tac(program: Program) -> Vec<TacInstr> {
    let mut result = Vec::new();

    assert_eq!(program.function.name, "main");

    let routine = generate_compound_stmt_tac(&program.function.body, &mut CodeEnv::new());
    result.extend(routine);

    // insert return 0 if no return is present
    let mut need_to_insert_return = true;
    if !result.is_empty() {
        if let TacInstr::Exit(_) = result.get(result.len() - 1).unwrap() {
            need_to_insert_return = false;
        }
    }
    if need_to_insert_return {
        result.push(TacInstr::Exit(TacVal::Lit(0)));
    }

    return result;
}

fn generate_compound_stmt_tac(stmts: &Vec<Statement>, code_env: &mut CodeEnv) -> Vec<TacInstr> {
    let mut result = Vec::new();

    let this_scopes_variable_map: HashMap<String, Identifier> = HashMap::new();
    code_env.var_map_list.push(this_scopes_variable_map);

    for statement in stmts {
        result.extend(generate_statement_tac(statement, code_env));
    }

    code_env.var_map_list.pop();

    result
}

fn generate_statement_tac(statement: &Statement, code_env: &mut CodeEnv) -> Vec<TacInstr> {
    match statement {
        Statement::Return(expr) => {
            let (mut result, expr_val) = generate_expr_tac(expr, code_env, None);
            result.push(TacInstr::Exit(expr_val));
            result
        }
        Statement::Declare(var_name, opt_value) => {
            let var_map_list = &mut code_env.var_map_list;
            let last_elem_index = var_map_list.len() - 1;
            let this_scopes_variable_map = var_map_list.get_mut(last_elem_index).unwrap();
            if this_scopes_variable_map.get(var_name).is_some() {
                panic!("doubly declared variable: {}", var_name);
            }
            let var_temp_loc = get_new_temp_name();

            match opt_value {
                Some(expr) => {
                    let (result, _) = generate_expr_tac(expr, code_env, Some(var_temp_loc));

                    let var_map_list = &mut code_env.var_map_list;
                    let last_elem_index = var_map_list.len() - 1;
                    let this_scopes_variable_map = var_map_list.get_mut(last_elem_index).unwrap();
                    this_scopes_variable_map.insert(var_name.clone(), var_temp_loc);

                    return result;
                }
                None => {
                    let var_map_list = &mut code_env.var_map_list;
                    let last_elem_index = var_map_list.len() - 1;
                    let this_scopes_variable_map = var_map_list.get_mut(last_elem_index).unwrap();
                    this_scopes_variable_map.insert(var_name.clone(), var_temp_loc);
                    return Vec::new();
                }
            };
        }
        Statement::Expr(expr) => {
            let (result, _) = generate_expr_tac(expr, code_env, None);
            result
        }
        Statement::Empty => {
            vec![]
        }
        Statement::CompoundStmt(stmts) => {
            let result = generate_compound_stmt_tac(stmts, code_env);
            result
        }
        Statement::If(condition, taken, not_taken) => {
            generate_if_statement_tac(condition, taken, not_taken.as_deref(), code_env)
        }
        Statement::While(condition, body) => gen_while_loop_tac(condition, body, code_env),
        Statement::Break => generate_break_tac(code_env),
        Statement::Continue => generate_continue_tac(code_env),
        Statement::For(initial_expr, control_expr, post_expr, body) => gen_for_loop_tac(
            initial_expr,
            control_expr.as_ref(),
            post_expr.as_ref(),
            body,
            code_env,
        ),
    }
}

fn resolve_variable_to_temp_name(name: &String, code_env: &CodeEnv) -> Identifier {
    for var_map in code_env.var_map_list.iter().rev() {
        if let Some(name) = var_map.get(name) {
            return *name;
        }
    }
    panic!("undeclared variable: {}", name);
}

fn generate_if_statement_tac(
    condition: &Expr,
    taken: &Statement,
    not_taken: Option<&Statement>,
    code_env: &mut CodeEnv,
) -> Vec<TacInstr> {
    let label_num = get_new_label_number();
    let label_not_taken = format!("if_not_taken_{}", label_num);
    let label_end = format!("if_end_{}", label_num);

    let (mut result, decision_val) = generate_expr_tac(condition, code_env, None);
    result.push(TacInstr::JmpZero(label_not_taken.clone(), decision_val));
    result.extend(generate_statement_tac(taken, code_env));
    result.push(TacInstr::Jmp(label_end.clone()));
    result.push(TacInstr::Label(label_not_taken));
    if let Some(not_taken) = not_taken {
        result.extend(generate_statement_tac(not_taken, code_env));
    }
    result.push(TacInstr::Label(label_end));

    result
}
