use crate::types::{FundT, VarType};

pub struct FunctionDecl {
    pub name: &'static str,
    pub return_type: VarType,
    pub num_args: usize,
}

pub const BUILTIN_FUNCTIONS: [FunctionDecl; 4] = [
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
        name: "puts",
        return_type: VarType::Fund(FundT::Int),
        num_args: 1,
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

pub fn generate_puts_asm() -> String {
    let result = "
.puts:
  mov rdx, 0
  mov r8, rdi
.begin_puts_loop:
  mov r9b, [r8]
  test r9b, r9b
  jz .end_puts_loop
  add r8, 1
  add rdx, 1
  jmp .begin_puts_loop
.end_puts_loop:
  add rdx, 1           ; rdx is the number of bytes to write
  mov byte [r8], 10    ; 10 is newline character code
  mov rsi, rdi
  mov rdi, 1           ; fd 1 for stdout
  mov rax, 1           ; syscall 1 for write
  syscall
  mov byte [r8], 0     ; put the null byte back
  ret
";
    return result.to_owned();
}
