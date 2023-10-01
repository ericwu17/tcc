#[derive(Debug)]
pub enum Token {
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    IntExpr { val: String },
    Identifier { val: String },
    Return,
    IntT,
    Semicolon,
}

pub fn get_tokens(source_code_contents: String) -> Vec<Token> {
    let mut cursor = source_code_contents.chars().peekable();

    let mut tokens: Vec<Token> = Vec::new();

    while cursor.peek().is_some() {
        // do something
        let next_char: char = *cursor.peek().unwrap();

        if next_char == '{' {
            cursor.next();
            tokens.push(Token::OpenBrace);
        } else if next_char == '}' {
            cursor.next();
            tokens.push(Token::CloseBrace);
        } else if next_char == '(' {
            cursor.next();
            tokens.push(Token::OpenParen);
        } else if next_char == ')' {
            cursor.next();
            tokens.push(Token::CloseParen);
        } else if next_char == ';' {
            cursor.next();
            tokens.push(Token::Semicolon);
        } else if next_char.is_ascii_whitespace() {
            // ignore all whitespace
            cursor.next();
        } else if next_char.is_digit(10) {
            let mut val = String::new();
            while cursor.peek().is_some() && (*cursor.peek().unwrap()).is_ascii_alphanumeric() {
                val.push(cursor.next().unwrap());
            }
            tokens.push(Token::IntExpr { val });
        } else if next_char.is_ascii_alphabetic() {
            let mut val = String::new();
            while cursor.peek().is_some() && (*cursor.peek().unwrap()).is_ascii_alphanumeric() {
                val.push(cursor.next().unwrap());
            }

            if val == "return" {
                tokens.push(Token::Return);
            } else if val == "int" {
                tokens.push(Token::IntT);
            } else {
                tokens.push(Token::Identifier { val });
            }
        } else {
            println!("you messed up, unrecognized character {}", next_char);
            std::process::exit(1);
        }
    }

    tokens
}
