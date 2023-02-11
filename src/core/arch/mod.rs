pub mod a6502;

/// A match pattern
pub enum Pattern {
    Exact(u8),
    And(u8),
    List(PatternList),
    Any,
}

impl Pattern {
    pub fn is_match(&self, byte: u8) -> bool {
        match self {
            Self::Exact(b) => *b == byte,
            Self::And(b) => *b & byte != 0,
            Self::List(l) => l.iter().fold(true, |i, p| i & p.is_match(byte)),
            Self::Any => true,
        }
    }
}

type PatternList = Vec<Pattern>;

pub struct ArchDef {
    patterns: Vec<PatternList>,
}
