use std::{
    fmt,
    hash::{Hash, Hasher},
};

use crate::types::{VarSize, VarType};

#[derive(Clone)]
pub struct Identifier(pub usize, pub VarType); // an identifier for a temporary in TAC

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Identifier {}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl fmt::Debug for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "temp{}for{}", self.0, self.1)
    }
}

impl Identifier {
    pub fn get_num_bytes(&self) -> usize {
        self.1.num_bytes()
    }

    pub fn get_size(&self) -> Option<VarSize> {
        return self.1.to_size();
    }
}
