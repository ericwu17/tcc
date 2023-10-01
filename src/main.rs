mod tokenizer;

use std::fs::File;
use std::io::Read;

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

    dbg!(tokens);
}
