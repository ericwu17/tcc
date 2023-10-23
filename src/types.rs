use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum VarType {
    Fund(FundT),
    Ptr(Box<VarType>),
    Arr(Box<VarType>, usize),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FundT {
    // a fundamental type
    Char,
    Short,
    Int,
    Long,
}

impl fmt::Display for VarType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VarType::Fund(inner) => write!(f, "{}", inner),
            VarType::Ptr(inner) => write!(f, "ptr to {}", inner),
            VarType::Arr(inner, len) => write!(f, "array of {} {}s", len, inner),
        }
    }
}

impl fmt::Display for FundT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FundT::Char => write!(f, "char"),
            FundT::Short => write!(f, "short"),
            FundT::Int => write!(f, "int"),
            FundT::Long => write!(f, "long"),
        }
    }
}

impl VarType {
    pub fn num_bytes(&self) -> usize {
        match self {
            VarType::Fund(inner) => inner.to_size().num_bytes(),
            VarType::Ptr(_) => 8,
            VarType::Arr(inner, len) => len * inner.num_bytes(),
        }
    }

    pub fn to_size(&self) -> Option<VarSize> {
        // returns None if this variable cannot be stored in a single register (arrays cannot be stored in a single register)
        match self {
            VarType::Fund(inner) => Some(inner.to_size()),
            VarType::Ptr(_) => Some(VarSize::Quad),
            VarType::Arr(_, _) => None,
        }
    }
}

impl FundT {
    pub fn to_size(&self) -> VarSize {
        match self {
            FundT::Char => VarSize::Byte,
            FundT::Short => VarSize::Word,
            FundT::Int => VarSize::Dword,
            FundT::Long => VarSize::Quad,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum VarSize {
    Byte,
    Word,
    Dword,
    Quad,
}

impl Default for VarSize {
    fn default() -> Self {
        VarSize::Dword
    }
}

impl VarSize {
    pub fn to_letter(&self) -> char {
        match self {
            VarSize::Byte => 'b',
            VarSize::Word => 'w',
            VarSize::Dword => 'd',
            VarSize::Quad => 'q',
        }
    }

    pub fn num_bytes(&self) -> usize {
        match self {
            VarSize::Byte => 1,
            VarSize::Word => 2,
            VarSize::Dword => 4,
            VarSize::Quad => 8,
        }
    }
}
