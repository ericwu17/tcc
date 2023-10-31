use crate::types::{FundT, VarType};

pub struct FunctionDecl {
    pub name: &'static str,
    pub return_type: VarType,
    pub num_args: usize,
}

pub const BUILTIN_FUNCTIONS: [FunctionDecl; 3] = [
    FunctionDecl {
        name: "putchar",
        return_type: VarType::Fund(FundT::Int),
        num_args: 1,
    },
    FunctionDecl {
        name: "getchar",
        return_type: VarType::Fund(FundT::Int),
        num_args: 0,
    },
    FunctionDecl {
        name: "exit",
        return_type: VarType::Fund(FundT::Int),
        num_args: 1,
    },
];

pub fn generate_putchar_asm() -> String {
    let result = "
.putchar:
  sub rsp, 1
  mov [rsp], dil
  mov rsi, rsp
  mov rdi, 1   ; stdout
  mov rdx, 1   ; 1 byte
  mov rax, 1   ; syscall #1 for 'write'
  syscall
  add rsp, 1
  ret
";

    return result.to_owned();
}

pub fn generate_getchar_asm() -> String {
    let result = "
.getchar:
  sub rsp, 4
  mov rsi, rsp ; a ptr to 'buf'
  mov rdi, 0   ; stdin
  mov rdx, 1   ; 1 byte to read
  mov rax, 0   ; syscall #0 for 'read'
  syscall
  test rax, rax          ; check for EOF (rax contains bytes written, returned by syscall)
  mov eax, [rsp]
  mov r10d, -1
  cmovz eax, r10d  ; if at EOF, return -1.
  add rsp, 4
  ret
";

    return result.to_owned();
}

pub fn generate_exit_asm() -> String {
    let result = "
.exit:
  mov eax, 231  ; syscall #231 for 'exit_group'
  syscall
";
    return result.to_owned();
}
