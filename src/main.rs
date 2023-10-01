mod parser;
mod tokenizer;
mod codegen;

use std::fs::File;
use std::io::Read;
use std::io::Write;

use codegen::generate_code;
use parser::generate_program_ast;
use tokenizer::get_tokens;

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

    let mut file = File::create("out.s").unwrap();
    file.write(asm_code.as_bytes()).unwrap();
}
