#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Op {
    Minus,
    BitwiseComplement,
    Not,
    Plus,
    Slash,
    Percent,
    DoubleAnd,
    DoublePipe,
    DoubleEq,
    NotEq,
    LessThan,
    GreaterThan,
    LessThanEq,
    GreaterThanEq,
    AssignmentEquals,
    PlusEquals,
    MinusEquals,
    MulEquals,
    DivEquals,
    ModEquals,
    PlusPlus,
    MinusMinus,
}

pub fn char_to_operator(c: char) -> Option<Op> {
    match c {
        '-' => Some(Op::Minus),
        '~' => Some(Op::BitwiseComplement),
        '!' => Some(Op::Not),
        '+' => Some(Op::Plus),
        '/' => Some(Op::Slash),
        '%' => Some(Op::Percent),
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
        ('+', '=') => Some(Op::PlusEquals),
        ('-', '=') => Some(Op::MinusEquals),
        ('*', '=') => Some(Op::MulEquals),
        ('/', '=') => Some(Op::DivEquals),
        ('%', '=') => Some(Op::ModEquals),
        ('+', '+') => Some(Op::PlusPlus),
        ('-', '-') => Some(Op::MinusMinus),
        _ => None,
    }
}
