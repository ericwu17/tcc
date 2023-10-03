#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
    Minus,
    BitwiseComplement,
    Not,
    Plus,
    Star,
    Slash,
    DoubleAnd,
    DoublePipe,
    DoubleEq,
    NotEq,
    LessThan,
    GreaterThan,
    LessThanEq,
    GreaterThanEq,
}

pub fn char_to_operator(c: char) -> Option<Op> {
    match c {
        '-' => Some(Op::Minus),
        '~' => Some(Op::BitwiseComplement),
        '!' => Some(Op::Not),
        '+' => Some(Op::Plus),
        '*' => Some(Op::Star),
        '/' => Some(Op::Slash),
        '<' => Some(Op::LessThan),
        '>' => Some(Op::GreaterThan),
        _ => None,
    }
}

pub fn chars_to_operator(chars: (char, char)) -> Option<Op> {
    match chars {
        ('&', '&') => Some(Op::DoubleAnd),
        ('|', '|') => Some(Op::DoublePipe),
        ('=', '=') => Some(Op::DoubleEq),
        ('!', '=') => Some(Op::NotEq),
        ('<', '=') => Some(Op::LessThanEq),
        ('>', '=') => Some(Op::GreaterThanEq),
        _ => None,
    }
}
