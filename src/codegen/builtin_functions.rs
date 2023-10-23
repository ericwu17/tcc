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
  sub rsp, 1
  mov rsi, rsp
  mov rdi, 0   ; stdin
  mov rdx, 1   ; 1 byte
  mov rax, 0   ; syscall #0 for 'read'
  syscall
  xor rax, rax
  mov al, [rsp]
  add rsp, 1
  ret
";

    return result.to_owned();
}
