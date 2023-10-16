pub mod operator;

use operator::{char_to_operator, chars_to_operator, Op};

use crate::{
    parser::expr_parser::{BinOp, BinOpPrecedenceLevel, UnOp},
    tac::VarSize,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    IntLit { val: String },
    Identifier { val: String },
    Return,
    Type(VarType),
    Semicolon,
    Comma,
    AssignmentEquals,
    Op(Op),
    QuestionMark,
    Colon,
    If,
    Else,
    While,
    For,
    Break,
    Continue,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum VarType {
    Char,
    Short,
    Int,
    Long,
}

impl VarType {
    pub fn to_size(&self) -> VarSize {
        match self {
            VarType::Char => VarSize::Byte,
            VarType::Short => VarSize::Word,
            VarType::Int => VarSize::Dword,
            VarType::Long => VarSize::Quad,
        }
    }
}

impl Token {
    pub fn to_binop_precedence_level(&self, level: BinOpPrecedenceLevel) -> Option<BinOp> {
        match level {
            BinOpPrecedenceLevel::MulDiv => match self {
                Token::Op(Op::Slash) => Some(BinOp::Divide),
                Token::Op(Op::Star) => Some(BinOp::Multiply),
                Token::Op(Op::Percent) => Some(BinOp::Modulus),
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
        } else if next_char == ',' {
            cursor.next();
            tokens.push(Token::Comma);
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
            // handle an integer literal
            let mut val = String::new();
            while cursor.peek().is_some() && (*cursor.peek().unwrap()).is_ascii_alphanumeric() {
                val.push(*cursor.next().unwrap());
            }
            tokens.push(Token::IntLit { val });
        } else if next_char.is_ascii_alphabetic() {
            // handle an identifier or C keyword
            let mut val = String::new();
            while cursor.peek().is_some() && (*cursor.peek().unwrap()).is_ascii_alphanumeric()
                || (*cursor.peek().unwrap()) == '_'
            {
                val.push(*cursor.next().unwrap());
            }

            match val.as_str() {
                "return" => tokens.push(Token::Return),
                "int" => tokens.push(Token::Type(VarType::Int)),
                "long" => tokens.push(Token::Type(VarType::Long)),
                "short" => tokens.push(Token::Type(VarType::Short)),
                "char" => tokens.push(Token::Type(VarType::Char)),
                "if" => tokens.push(Token::If),
                "else" => tokens.push(Token::Else),
                "while" => tokens.push(Token::While),
                "break" => tokens.push(Token::Break),
                "continue" => tokens.push(Token::Continue),
                "for" => tokens.push(Token::For),
                _ => tokens.push(Token::Identifier { val }),
            }
        } else if next_char == '\'' {
            cursor.next(); // consume the single quote char

            let mut val = String::new();
            while cursor.peek().is_some()
                && (*cursor.peek().unwrap()) != '\''
                && (*cursor.peek().unwrap()) != '\n'
            {
                val.push(*cursor.next().unwrap());
                if val.ends_with('\\') {
                    val.push(*cursor.next().unwrap());
                }
            }
            if cursor.next() != Some(&'\'') {
                panic!("expected a closing `'` for character expression!")
            }

            tokens.push(Token::IntLit {
                val: convert_str_to_char_int(val),
            })
        } else {
            println!("you messed up, unrecognized character {}", next_char);
            std::process::exit(1);
        }
    }

    tokens
}

fn convert_str_to_char_int(val: String) -> String {
    match val.len() {
        1 => {
            let res = val.chars().next().unwrap();
            format!("{}", res as i32)
        }
        2 => {
            assert_eq!(val.chars().nth(0).unwrap(), '\\');

            match val.chars().nth(1).unwrap() {
                't' => "9".to_owned(),
                'n' => "10".to_owned(),
                '\\' => "92".to_owned(),
                '0' => "0".to_owned(),
                '\'' => "39".to_owned(),
                _ => {
                    panic!("unrecognized character escape sequence: '{}'", val)
                }
            }
        }

        _ => {
            panic!("invalid length for character literal");
        }
    }
}
