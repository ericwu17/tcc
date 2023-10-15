pub fn generate_putchar_asm() -> String {
    let result = "
putchar:
  add rsp, 8
  mov [rsp], edi
  mov rsi, rsp
  mov rdi, 1   ; stdout
  mov rdx, 1   ; 1 byte
  mov rax, 1   ; syscall #1
  syscall
  sub rsp, 8
  ret
";

    return result.to_owned();
}
