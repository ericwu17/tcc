mod codegen;
mod parser;
mod tokenizer;

use std::fs::File;
use std::io::Read;
use std::io::Write;

use std::process::Command;

use codegen::generate_code;
use parser::generate_program_ast;
use tokenizer::get_tokens;

const ASM_FILE_NAME: &str = "out.asm";
const OBJ_FILE_NAME: &str = "out.o";
const EXEC_FILE_NAME: &str = "a.out";

fn main() {
    let input_filepath = std::env::args().nth(1).expect("Please give a filename");

    let mut input_file =
        File::open(&input_filepath).expect(&format!("could not open file: {}", &input_filepath));
    let mut contents = String::new();
    input_file
        .read_to_string(&mut contents)
        .expect(&format!("error reading file: {}", &input_filepath));

    let tokens = get_tokens(contents);

    let program_AST = generate_program_ast(&mut tokens.into_iter().peekable());

    let asm_code: String = generate_code(program_AST);

    let mut file = File::create(ASM_FILE_NAME).unwrap();
    file.write(asm_code.as_bytes()).unwrap();

    assemble_and_link();
}

fn assemble_and_link() {
    Command::new("nasm")
        .args(["-f", "elf64"])
        .arg(ASM_FILE_NAME)
        .args(["-o", OBJ_FILE_NAME])
        .output()
        .expect("failed to execute assembler process");

    Command::new("ld")
        .arg(OBJ_FILE_NAME)
        .args(["-o", EXEC_FILE_NAME])
        .output()
        .expect("failed to execute linker process");

    Command::new("rm")
        .arg(OBJ_FILE_NAME)
        .output()
        .expect("failed to execute process to remove object file");
}
