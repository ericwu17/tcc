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
  cmovz eax, r10d  ; if at EOF, write -1 to memory.
  add rsp, 4
  ret
";

    return result.to_owned();
}
