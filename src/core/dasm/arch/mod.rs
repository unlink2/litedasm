#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::FdResult;

use super::{
    symbols::{Symbol, SymbolKey, SymbolList},
    Address,
};

type DisasCallback<T> = fn(text: &str, ctx: &Context, u: T) -> FdResult<()>;

pub fn default_callback(text: &str, ctx: &Context, usr: &mut dyn std::io::Write) -> FdResult<()> {
    todo!()
}

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
/// TODO implement transforms for all other possible data types
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub enum Transform {
    /// The absolute formatters take in the input array,
    /// and attempt to read its absolute value at a given offset
    /// from the array
    AbsU8Hex(usize),
    AbsU16Hex(usize),
    AbsU32Hex(usize),
    AbsU64Hex(usize),
    String(String),
    /// Defsym takes a string and defines a clear name
    /// for the given value at the requested offset
    DefSymU8(String, usize),
    DefSymU16(String, usize),
    DefSymU32(String, usize),
    DefSymU64(String, usize),
    DefSymAddress(String, usize),
    #[default]
    Skip,
}

impl Transform {
    pub fn apply<T>(
        &self,
        f: DisasCallback<T>,
        data: &[u8],
        ctx: &mut Context,
        u: T,
    ) -> FdResult<usize> {
        todo!()
    }
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
#[derive(Default, Clone)]
pub struct Context {
    org: Address,
    syms: SymbolList,
}

impl Context {
    pub fn disas<T>(&mut self, f: DisasCallback<T>, data: &[u8], u: T) -> FdResult<()> {
        todo!()
    }

    pub fn def_sym(&mut self, key: SymbolKey, sym: Symbol) {
        self.syms.def_symbol(key, sym);
    }

    pub fn get_sym(&self, key: SymbolKey) -> Option<&Symbol> {
        self.syms.get_symbol(key)
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct ArchDef {
    patterns: Vec<PatternList>,
    endianess: Endianess,
    // size of address in bytes for the given architecture
    addr_size: usize,
}

impl ArchDef {
    /// start disasssembly
    /// This will write all result strings to the f callback,
    /// and it will modify the current context
    pub fn disas<T>(
        &self,
        f: DisasCallback<T>,
        data: &[u8],
        ctx: &mut Context,
        u: T,
    ) -> FdResult<()> {
        Ok(())
    }
}
