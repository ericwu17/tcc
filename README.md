# tcc

This is a personal project to learn more about compilers and the x86-64 architecture.
tcc (toy-c-compiler) can compile a tiny subset of C programs into asm files.
tcc will then assemble the file using `nasm` and link the output with `ld`. Currently,
the compiler's goal is only to generate correct code -- it does not aim to be efficient or
optimized.

Implemented features include:

- variable declarations and assignments of type `long`, `int`, `short`, and `char`
- for and while loops, if statements
- stdout and stdin using the `putchar()` and `getchar()` functions
- function definitions and function calls

Future plans include:

- pointers, arrays, and static strings (so that `puts()` may be used instead of repeated `putchar()` calls)
- C structs
- `malloc()` and `free()` (I might want to implement these in C code, so this might involve making my own subset of the C standard library)
- floats and doubles!
- more efficient code using techniques such as register allocation, dead code elimination, loop optimizations...

The asm output is generated in the file `out.asm` which is ignored by git.

## Tests

Tests are especially helpful to catch regressions in a compiler. Tests can be run through the command
`cargo test`. There two sets of tests: one which compiles valid programs in `/tests/programs` and asserts that tcc's
compiled program's output matches a binary compiled by gcc, and another which asks tcc to compile
a set of invalid programs in `/tests/programs_invalid`, and asserts that tcc fails to compile them.

Many, but not all, of the tests came from Nora Sandler's blog where she provides a test for small C compilers:
https://github.com/nlsandler/write_a_c_compiler

## Running the compiler

The compiler can be run using cargo by the command `cargo run test.c` where `test.c` contains
C source code to be compiled.

The compiler also supports two flags:

- `-d` enables printing of debug information such as the token stream, abstract syntax tree, and three-address intermediate representation
- `-n` skips the assembly and link stage, which is helpful when running the compiler on a non-x86-64 computer.

## References used

I have found many links helpful while writing this compiler:

- Nora Sandler's blog on writing a C compiler using OCaml: https://norasandler.com/2017/11/29/Write-a-Compiler.html
- A YouTube series by Pixeled about creating a compiler using C++: https://www.youtube.com/watch?v=vcSijrRsrY0

These references have helped me learn about parsing strategies, and have motivated me to continue this project.

## Example program compiled with tcc

Here is a sample program, which computes the 7th fibonacci number (13)
and returns it as the exit code.

```C
int fibb(int n) {
    if (n == 0) {
        return 0;
    }
    if (n == 1) {
        return 1;
    }
    return fibb(n-1) + fibb(n-2);
}

int main() {
    return fibb(7);
}
```

tcc generates the following assembly code:

```asm
global _start
.fibb:
  push rbp
  mov rbp, rsp
  sub rsp, 32
  mov [rbp - 4], edi
  mov edi, [rbp - 4]
  movsx rdi, edi
  mov esi, 0
  cmp edi, esi
  mov rdi, 0
  sete dil
  mov [rbp - 8], edi
  mov edi, [rbp - 8]
  movsx rdi, edi
  test edi, edi
  je .if_not_taken_0
  mov eax, 0
  mov rsp, rbp
  pop rbp
  ret
  jmp .if_end_0
.if_not_taken_0:
.if_end_0:
  mov edi, [rbp - 4]
  movsx rdi, edi
  mov esi, 1
  cmp edi, esi
  mov rdi, 0
  sete dil
  mov [rbp - 12], edi
  mov edi, [rbp - 12]
  movsx rdi, edi
  test edi, edi
  je .if_not_taken_1
  mov eax, 1
  mov rsp, rbp
  pop rbp
  ret
  jmp .if_end_1
.if_not_taken_1:
.if_end_1:
  mov edi, [rbp - 4]
  movsx rdi, edi
  mov esi, 1
  sub edi, esi
  mov [rbp - 16], edi
  mov edi, [rbp - 16]
  movsx rdi, edi
  call .fibb
  mov [rbp - 20], eax
  mov edi, [rbp - 4]
  movsx rdi, edi
  mov esi, 2
  sub edi, esi
  mov [rbp - 24], edi
  mov edi, [rbp - 24]
  movsx rdi, edi
  call .fibb
  mov [rbp - 28], eax
  mov edi, [rbp - 20]
  movsx rdi, edi
  mov esi, [rbp - 28]
  movsx rsi, esi
  add edi, esi
  mov [rbp - 32], edi
  mov eax, [rbp - 32]
  movsx rax, eax
  mov rsp, rbp
  pop rbp
  ret
_start:
  push rbp
  mov rbp, rsp
  sub rsp, 4
  mov edi, 7
  call .fibb
  mov [rbp - 4], eax
  mov edi, [rbp - 4]
  movsx rdi, edi
  mov eax, 231
  syscall
```

this example demonstrates the possibility for future improvements to the machine code
omitted by tcc.
