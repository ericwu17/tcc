use crate::parser::{BinOp, UnOp};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
    Minus,
    BitwiseComplement,
    Not,
    Plus,
    Star,
    Slash,
}

pub fn is_operator(c: &char) -> bool {
    return ['-', '~', '!', '+', '*', '/'].contains(c);
}

pub fn char_to_operator(c: &char) -> Op {
    match c {
        '-' => Op::Minus,
        '~' => Op::BitwiseComplement,
        '!' => Op::Not,
        '+' => Op::Plus,
        '*' => Op::Star,
        '/' => Op::Slash,
        _ => {
            unreachable!()
        }
    }
}
