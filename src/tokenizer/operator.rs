use crate::parser::UnOp;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
    Minus,
    BitwiseComplement,
    Not,
    Plus,
    Star,
    Slash,
}

impl Op {
    pub fn to_un_op(&self) -> Option<UnOp> {
        match self {
            Op::Minus => Some(UnOp::Negation),
            Op::BitwiseComplement => Some(UnOp::BitwiseComplement),
            Op::Not => Some(UnOp::Not),
            _ => None,
        }
    }
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
