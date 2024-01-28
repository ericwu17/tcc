use crate::types::{FundT, VarType};

pub struct FunctionDecl {
    pub name: &'static str,
    pub return_type: VarType,
    pub num_args: usize,
    pub asm_code: &'static str,
}

pub const BUILTIN_FUNCTIONS: [FunctionDecl; 5] = [
    FunctionDecl {
        name: "putchar",
        return_type: VarType::Fund(FundT::Int),
        num_args: 1,
        asm_code: generate_putchar_asm(),
    },
    FunctionDecl {
        name: "getchar",
        return_type: VarType::Fund(FundT::Int),
        num_args: 0,
        asm_code: generate_getchar_asm(),
    },
    FunctionDecl {
        name: "puts",
        return_type: VarType::Fund(FundT::Int),
        num_args: 1,
        asm_code: generate_puts_asm(),
    },
    FunctionDecl {
        name: "strlen",
        return_type: VarType::Fund(FundT::Int),
        num_args: 1,
        asm_code: generate_strlen_asm(),
    },
    FunctionDecl {
        name: "exit",
        return_type: VarType::Fund(FundT::Int),
        num_args: 1,
        asm_code: generate_exit_asm(),
    },
];

const fn generate_putchar_asm() -> &'static str {
    // todo:
    ""
}

const fn generate_getchar_asm() -> &'static str {
    // todo:
    ""
}

const fn generate_exit_asm() -> &'static str {
    // todo:
    ""
}

const fn generate_puts_asm() -> &'static str {
    // todo:
    ""
}

const fn generate_strlen_asm() -> &'static str {
    // todo:
    ""
}
