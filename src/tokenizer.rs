pub mod operator;

use operator::{char_to_operator, chars_to_operator, Op};

use crate::parser::expr_parser::{BinOp, BinOpPrecedenceLevel, UnOp};

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    IntLit { val: String },
    Identifier { val: String },
    Return,
    IntT,
    Semicolon,
    AssignmentEquals,
    Op(Op),
    QuestionMark,
    Colon,
    If,
    Else,
}

impl Token {
    pub fn to_binop_precedence_level(&self, level: BinOpPrecedenceLevel) -> Option<BinOp> {
        match level {
            BinOpPrecedenceLevel::MulDiv => match self {
                Token::Op(Op::Slash) => Some(BinOp::Divide),
                Token::Op(Op::Star) => Some(BinOp::Multiply),
                _ => None,
            },
            BinOpPrecedenceLevel::AddSub => match self {
                Token::Op(Op::Minus) => Some(BinOp::Minus),
                Token::Op(Op::Plus) => Some(BinOp::Plus),
                _ => None,
            },
            BinOpPrecedenceLevel::OrderingCmp => match self {
                Token::Op(Op::LessThan) => Some(BinOp::LessThan),
                Token::Op(Op::LessThanEq) => Some(BinOp::LessThanEq),
                Token::Op(Op::GreaterThan) => Some(BinOp::GreaterThan),
                Token::Op(Op::GreaterThanEq) => Some(BinOp::GreaterThanEq),
                _ => None,
            },
            BinOpPrecedenceLevel::EqCmp => match self {
                Token::Op(Op::NotEq) => Some(BinOp::NotEquals),
                Token::Op(Op::DoubleEq) => Some(BinOp::Equals),
                _ => None,
            },
            BinOpPrecedenceLevel::LogicalAnd => match self {
                Token::Op(Op::DoubleAnd) => Some(BinOp::LogicalAnd),
                _ => None,
            },
            BinOpPrecedenceLevel::LogicalOr => match self {
                Token::Op(Op::DoublePipe) => Some(BinOp::LogicalOr),
                _ => None,
            },
        }
    }

    pub fn to_un_op(&self) -> Option<UnOp> {
        match self {
            Token::Op(Op::Minus) => Some(UnOp::Negation),
            Token::Op(Op::BitwiseComplement) => Some(UnOp::BitwiseComplement),
            Token::Op(Op::Not) => Some(UnOp::Not),
            _ => None,
        }
    }
}

pub struct SourceCodeCursor {
    contents: Vec<char>,
    index: usize,
}

impl SourceCodeCursor {
    fn new(contents: String) -> Self {
        SourceCodeCursor {
            contents: contents.chars().collect(),
            index: 0,
        }
    }

    fn peek(&self) -> Option<&char> {
        self.contents.get(self.index)
    }
    fn peek_nth(&self, n: usize) -> Option<&char> {
        // peek_nth(1) is equivalent to peek()
        self.contents.get(self.index + n - 1)
    }

    fn next(&mut self) -> Option<&char> {
        self.index += 1;
        self.contents.get(self.index - 1)
    }
}

pub fn get_tokens(source_code_contents: String) -> Vec<Token> {
    let mut cursor = SourceCodeCursor::new(source_code_contents);

    let mut tokens: Vec<Token> = Vec::new();

    while cursor.peek().is_some() {
        let next_char: char = *cursor.peek().unwrap();
        let next_next_char: char = *cursor.peek_nth(2).unwrap_or(&' ');

        if next_char == '/' && cursor.peek_nth(2) == Some(&'/') {
            // ignore single line comments
            while cursor.peek().is_some() && cursor.next() != Some(&'\n') {}
        } else if next_char == '{' {
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
        } else if next_char == ':' {
            cursor.next();
            tokens.push(Token::Colon);
        } else if next_char == '?' {
            cursor.next();
            tokens.push(Token::QuestionMark);
        } else if let Some(op) = chars_to_operator((next_char, next_next_char)) {
            // must consume 2 characters for an operator that is 2 characters long
            cursor.next();
            cursor.next();
            tokens.push(Token::Op(op));
        } else if next_char == '=' {
            cursor.next();
            tokens.push(Token::AssignmentEquals);
        } else if let Some(op) = char_to_operator(next_char) {
            cursor.next();
            tokens.push(Token::Op(op));
        } else if next_char.is_ascii_whitespace() {
            // ignore all whitespace
            cursor.next();
        } else if next_char.is_digit(10) {
            let mut val = String::new();
            while cursor.peek().is_some() && (*cursor.peek().unwrap()).is_ascii_alphanumeric() {
                val.push(*cursor.next().unwrap());
            }
            tokens.push(Token::IntLit { val });
        } else if next_char.is_ascii_alphabetic() {
            let mut val = String::new();
            while cursor.peek().is_some() && (*cursor.peek().unwrap()).is_ascii_alphanumeric() {
                val.push(*cursor.next().unwrap());
            }

            if val == "return" {
                tokens.push(Token::Return);
            } else if val == "int" {
                tokens.push(Token::IntT);
            } else if val == "if" {
                tokens.push(Token::If)
            } else if val == "else" {
                tokens.push(Token::Else)
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
