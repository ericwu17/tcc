use std::collections::HashMap;
use std::fmt;

pub mod generation;
pub mod tac_func;
pub mod tac_instr;

use crate::errors::check_funcs::check_funcs;
use crate::errors::check_types::check_types;
use crate::errors::check_vars::check_vars;
use crate::parser::expr_parser::ExprEnum;
use crate::parser::Function;
use crate::parser::{expr_parser::Expr, Program, Statement};
use crate::types::{VarSize, VarType};

use self::generation::binop::generate_binop_tac;
use self::generation::break_stmt::generate_break_stmt_code;
use self::generation::continue_stmt::generate_continue_stmt_code;
use self::generation::declare::generate_declaration_tac;
use self::generation::for_loop::generate_for_loop_tac;
use self::generation::if_stmt::generate_if_statement_tac;
use self::generation::while_loop::generate_while_loop_tac;
use self::tac_func::{BBIdentifier, TacFunc};
use self::tac_instr::{TacBBInstr, TacBasicBlock, TacTransitionInstr};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct Identifier(usize, VarSize); // an identifier for a temporary in TAC, represents a offset from RBP

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = self.1.to_letter();
        write!(f, "{}{}", suffix, self.0)
    }
}

impl Identifier {
    pub fn get_num_bytes(&self) -> usize {
        match self.1 {
            VarSize::Byte => 1,
            VarSize::Word => 2,
            VarSize::Dword => 4,
            VarSize::Quad => 8,
        }
    }

    pub fn get_size(&self) -> VarSize {
        self.1
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum TacVal {
    Lit(i64, VarSize),
    Var(Identifier),
}

impl TacVal {
    pub fn get_size(&self) -> VarSize {
        match self {
            TacVal::Lit(_, size) => *size,
            TacVal::Var(ident) => ident.1,
        }
    }
}

impl fmt::Debug for TacVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TacVal::Lit(val, var_size) => write!(f, "{}{}", val, var_size.to_letter()),
            TacVal::Var(ident) => write!(f, "{:?}", ident),
        }
    }
}

pub struct CodeEnv {
    /// a list of maps, one for each source code scope level, mapping variable names to temporary storage names
    pub var_map_list: Vec<HashMap<String, Identifier>>,

    /// A way to identify the current basic block being modified.
    pub current_bb: BBIdentifier,

    /// Jump to here upon break statement
    pub loop_label_end: Option<BBIdentifier>,
    /// Jump to here upon continue statement
    pub loop_label_begin: Option<BBIdentifier>,
    /// The next number to be assigned for a new temporary
    pub temp_storage_number: usize,
}

pub struct TacGenerator<'a> {
    curr_context: CodeEnv,
    current_output: TacFunc,
    ast: &'a Function,
}

impl<'a> TacGenerator<'a> {
    pub fn new(ast: &'a Function) -> Self {
        let mut code_env = CodeEnv::new();
        let mut this_scopes_variable_map: HashMap<String, Identifier> = HashMap::new();
        let mut args = Vec::new();

        for (arg_name, arg_type) in ast.args.iter() {
            let var_temp_loc = code_env.get_new_temp_name(arg_type.to_size().unwrap());
            this_scopes_variable_map.insert(arg_name.clone(), var_temp_loc);
            args.push((var_temp_loc, arg_type.clone()));
        }
        code_env.var_map_list.push(this_scopes_variable_map);

        let unfinished_bb = TacBasicBlock::new(0);

        let new_func = TacFunc {
            name: ast.name.clone(),
            args,
            basic_blocks: vec![unfinished_bb],
        };

        TacGenerator {
            curr_context: code_env,
            current_output: new_func,
            ast,
        }
    }

    pub fn generate_tac(mut self) -> TacFunc {
        for stmt in &self.ast.body {
            self.consume_statement(stmt);
        }

        self.current_output
    }

    pub fn consume_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Continue => generate_continue_stmt_code(self),
            Statement::Break => generate_break_stmt_code(self),
            Statement::Return(expr) => {
                let ident = self.consume_expr(expr, None);
                self.set_curr_bb_out_instr(TacTransitionInstr::Return(TacVal::Var(ident)));
            }
            Statement::Declare(var_name, opt_expr, var_type) => {
                generate_declaration_tac(self, var_name, opt_expr, var_type);
            }
            Statement::CompoundStmt(stmts) => {
                let this_scopes_variable_map: HashMap<String, Identifier> = HashMap::new();
                self.curr_context
                    .var_map_list
                    .push(this_scopes_variable_map);
                for stmt in stmts {
                    self.consume_statement(stmt);
                }
                self.curr_context.var_map_list.pop();
            }
            Statement::If(ctrl_expr, taken, opt_not_taken) => {
                generate_if_statement_tac(self, ctrl_expr, taken, opt_not_taken.as_deref());
            }
            Statement::While(ctrl_expr, loop_body) => {
                generate_while_loop_tac(self, ctrl_expr, loop_body);
            }
            Statement::For(init_stmt, ctrl_expr, post_expr, loop_body) => {
                generate_for_loop_tac(
                    self,
                    init_stmt,
                    ctrl_expr.as_ref(),
                    post_expr.as_ref(),
                    loop_body,
                );
            }
            Statement::Expr(expr) => {
                self.consume_expr(expr, None);
            }
            Statement::Empty => {}
        }
    }

    pub fn consume_expr(&mut self, expr: &Expr, size: Option<VarSize>) -> Identifier {
        match &expr.content {
            ExprEnum::Int(x) => {
                let var_size = get_expr_size(expr).unwrap_or(size.unwrap_or_default());
                let ident = self.get_new_temp_name(var_size);
                self.push_instr(TacBBInstr::Copy(ident, TacVal::Lit(*x, var_size)));

                ident
            }
            ExprEnum::Var(var_name) => self.curr_context.resolve_variable_to_temp_name(&var_name),
            ExprEnum::UnOp(op, inner_expr) => {
                let ident = self.consume_expr(&inner_expr, size);
                let new_ident = self.get_new_temp_name(size.unwrap_or_default());
                self.push_instr(TacBBInstr::UnOp(new_ident, TacVal::Var(ident), *op));

                new_ident
            }
            ExprEnum::BinOp(op, expr1, expr2) => generate_binop_tac(self, *op, expr1, expr2, size),
            ExprEnum::Ternary(_, _, _) => todo!(),
            ExprEnum::FunctionCall(func_name, args) => {
                let mut arg_idents = Vec::new();
                for arg_expr in args {
                    arg_idents.push(self.consume_expr(arg_expr, None));
                }
                let arg_idents = arg_idents.iter().map(|id| TacVal::Var(*id)).collect();

                let new_ident = self.get_new_temp_name(size.unwrap_or_default());

                self.push_instr(TacBBInstr::Call(new_ident, func_name.clone(), arg_idents));

                new_ident
            }
            ExprEnum::Deref(_) => todo!(),
            ExprEnum::Ref(_) => todo!(),
            ExprEnum::PostfixDec(_) => todo!(),
            ExprEnum::PostfixInc(_) => todo!(),
            ExprEnum::PrefixDec(_) => todo!(),
            ExprEnum::PrefixInc(_) => todo!(),
            ExprEnum::Sizeof(_) => todo!(),
            ExprEnum::ArrInitExpr(_) => todo!(),
            ExprEnum::StaticStrPtr(_) => todo!(),
        }
    }
    fn get_curr_bb(&mut self) -> &mut TacBasicBlock {
        let curr_bb_index = self.curr_context.current_bb;
        &mut self.current_output.basic_blocks[curr_bb_index]
    }

    fn get_new_temp_name(&mut self, size: VarSize) -> Identifier {
        self.curr_context.get_new_temp_name(size)
    }

    pub fn push_instr(&mut self, instr: TacBBInstr) {
        if self.get_curr_bb().out_instr == TacTransitionInstr::Null {
            self.get_curr_bb().instrs.push(instr);
        } else {
            println!("WARNING: unreachable expression (ignoring it)");
        }
    }

    pub fn set_curr_bb_out_instr(&mut self, instr: TacTransitionInstr) {
        if self.get_curr_bb().out_instr == TacTransitionInstr::Null {
            self.get_curr_bb().out_instr = instr;
        }
    }
}

impl CodeEnv {
    pub fn new() -> Self {
        CodeEnv {
            var_map_list: Vec::new(),
            current_bb: 0,
            loop_label_end: None,
            loop_label_begin: None,
            temp_storage_number: 0,
        }
    }

    pub fn get_new_temp_name(&mut self, size: VarSize) -> Identifier {
        let n = self.temp_storage_number;
        self.temp_storage_number += 1;
        Identifier(n, size)
    }
    pub fn resolve_variable_to_temp_name(&self, name: &str) -> Identifier {
        for var_map in self.var_map_list.iter().rev() {
            if let Some(name) = var_map.get(name) {
                return *name;
            }
        }
        // unreachable because check_vars should have already checked that each variable was declared properly.
        panic!("CodeEnv tried to resolve a bad variable name")
    }
}

/// This function takes as input a program AST,
/// and as output will generate the TAC IR (three-address-code intermediate representation),
/// where each TacFunc object is the IR for a single function.
pub fn generate_tac(mut program: Program) -> Vec<TacFunc> {
    check_funcs(&program);
    check_vars(&program);
    check_types(&mut program); // check types will also evaluate sizeof, thus we need mut

    let mut tac_funcs = Vec::new();

    for function in &program.functions {
        let tac_gen = TacGenerator::new(function);
        tac_funcs.push(tac_gen.generate_tac());
    }

    tac_funcs
}

fn get_expr_size(expr: &Expr) -> Option<VarSize> {
    let t = expr.type_.clone()?;
    Some(get_type_size(&t))
}

fn get_type_size(t: &VarType) -> VarSize {
    if let VarType::Arr(_, _) = t {
        // arrays are pointers, and therefore occupy a quad
        return VarSize::Quad;
    }
    match t.num_bytes() {
        1 => VarSize::Byte,
        2 => VarSize::Word,
        4 => VarSize::Dword,
        8 => VarSize::Quad,
        _ => unreachable!(),
    }
}
