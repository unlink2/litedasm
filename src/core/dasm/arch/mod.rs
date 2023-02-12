pub mod a6502;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{symbols::SymbolList, Address};

/// A match pattern
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub enum Pattern {
    Exact(u8),
    And(u8),
    List(PatternList),
    Any,
    #[default]
    Never,
}

impl Pattern {
    pub fn is_match(&self, byte: u8) -> bool {
        match self {
            Self::Exact(b) => *b == byte,
            Self::And(b) => *b & byte != 0,
            Self::List(l) => l.iter().fold(true, |i, p| i & p.is_match(byte)),
            Self::Any => true,
            Self::Never => false,
        }
    }
}

type PatternList = Vec<Pattern>;

/// A formatter takes an input &[u8] and applies a transform the the data
/// then it outputs its contents to anything with a dyn Write trait  
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub enum Transform {
    /// The absolute formatters take in the input array,
    /// and attempt to
    AbsU8Hex(usize),
    AbsU16Hex(usize),
    AbsU32Hex(usize),
    AbsU64Hex(usize),
    String(String),
    // Defsym takes a string and defines a clear name
    // for the given value at the requested offset
    DefSymU8(String, usize),
    DefSymU16(String, usize),
    DefSymU32(String, usize),
    DefSymU64(String, usize),
    #[default]
    Skip,
}

/// A matcher matches the pattern list and if *all* patterns match
/// it will apply the formatter
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct Matcher {
    patterns: Vec<PatternList>,
    transforms: Vec<Transform>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub enum Endianess {
    // controversial default
    #[default]
    Little,
    Big,
}

/// The context describes the runtime information of a single parser operation
/// it contains the current address as well as a list of known symbols
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct Context {
    org: Address,
    endianess: Endianess,
    syms: SymbolList,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct ArchDef {
    patterns: Vec<PatternList>,
}

impl ArchDef {}
