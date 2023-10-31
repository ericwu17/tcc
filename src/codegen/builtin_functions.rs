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
    "
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
"
}

const fn generate_getchar_asm() -> &'static str {
    "
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
"
}

const fn generate_exit_asm() -> &'static str {
    "
.exit:
  mov eax, 231  ; syscall #231 for 'exit_group'
  syscall
"
}

const fn generate_puts_asm() -> &'static str {
    "
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
"
}

const fn generate_strlen_asm() -> &'static str {
    "
.strlen:
  mov rax, 0
.begin_strlen_loop:
  mov r9b, [rdi]
  test r9b, r9b
  jz .end_strlen_loop
  add rax, 1
  add rdi, 1
  jmp .begin_strlen_loop
.end_strlen_loop:
  ret
"
}
