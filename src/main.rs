mod codegen;
mod parser;
mod tac;
mod tokenizer;

use std::fs::File;
use std::io::Read;
use std::io::Write;

use std::process::Command;

use clap::Parser;
use codegen::generate_x86_code;
use parser::generate_program_ast;
use tac::generate_tac;
use tokenizer::get_tokens;

use crate::codegen::asm_gen::generate_program_asm;

const ASM_FILE_NAME: &str = "out.asm";
const OBJ_FILE_NAME: &str = "out.o";
const EXEC_FILE_NAME: &str = "a.out";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Name of input file containing C source code
    filename: String,
    /// Skip the assembly and link step of compilation, only generating the assembly file
    #[arg(short = 'n', long = "no-assemble")]
    no_assemble: bool,
    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();

    let input_filepath = cli.filename;
    let no_assemble = cli.no_assemble;

    let mut contents = String::new();
    File::open(&input_filepath)
        .expect(&format!("could not open file: {}", &input_filepath))
        .read_to_string(&mut contents)
        .expect(&format!("error reading file: {}", &input_filepath));

    let tokens = get_tokens(contents);
    if cli.debug {
        dbg!(&tokens);
    }
    let program_ast = generate_program_ast(tokens);
    if cli.debug {
        dbg!(&program_ast);
    }

    let tac_ir = generate_tac(program_ast);
    if cli.debug {
        dbg!(&tac_ir);
    }

    let x86_code = generate_x86_code(&tac_ir);
    if cli.debug {
        dbg!(&x86_code);
    }

    let asm_code = generate_program_asm(&x86_code);

    File::create(ASM_FILE_NAME)
        .expect("error creating ASM output file.")
        .write(asm_code.as_bytes())
        .expect("error writing output to ASM output file.");

    if !no_assemble {
        assemble_and_link();
    }
}

fn assemble_and_link() {
    let output = Command::new("nasm")
        .args(["-g", "-f", "elf64"])
        .arg(ASM_FILE_NAME)
        .args(["-o", OBJ_FILE_NAME])
        .output()
        .expect("failed to execute assembler process");
    if output.status.code() != Some(0) {
        dbg!(output);
        panic!();
    }

    let output = Command::new("ld")
        .arg(OBJ_FILE_NAME)
        .args(["-o", EXEC_FILE_NAME])
        .output()
        .expect("failed to execute linker process");
    if output.status.code() != Some(0) {
        dbg!(output);
        panic!();
    }

    let output = Command::new("rm")
        .arg(OBJ_FILE_NAME)
        .output()
        .expect("failed to execute process to remove object file");
    if output.status.code() != Some(0) {
        dbg!(output);
        panic!();
    }
}
