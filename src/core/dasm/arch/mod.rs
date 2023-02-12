#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Error, FdResult};

use super::{
    symbols::{Symbol, SymbolKey, SymbolKind, SymbolList},
    Address,
};

/// This callback is called for every matched pattern with the final
/// transformed result. Each context may also pass along a user-data field <T>
/// which can be used to make the callback work
pub trait DisasCallback<T> = FnMut(&str, &ArchDef, &mut Context, T) -> FdResult<()>;

pub fn default_callback(
    text: &str,
    arch: &ArchDef,
    ctx: &mut Context,
    usr: &mut dyn std::io::Write,
) -> FdResult<()> {
    todo!()
}

/// A match pattern
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
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

/// A formatter takes an input &[u8] and applies a transform to the data
/// then it outputs its contents to anything with a dyn Write trait  
/// TODO implement transforms for all other possible data types
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
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
        mut f: impl DisasCallback<T>,
        data: &[u8],
        arch: &ArchDef,
        ctx: &mut Context,
        u: T,
    ) -> FdResult<usize> {
        // get all data, if no data is available just return with an error
        // since a transform should *never* be out of data
        // assuming the pattern is defined correctly!
        let data = Self::get_data(data, self.offset(), self.data_len(arch))
            .ok_or(Error::TransformOutOfData(ctx.org))?;

        match self {
            Transform::AbsU8Hex(_) => todo!(),
            Transform::AbsU16Hex(_) => todo!(),
            Transform::AbsU32Hex(_) => todo!(),
            Transform::AbsU64Hex(_) => todo!(),
            Transform::String(s) => f(s, arch, ctx, u)?,
            Transform::DefSymU8(s, _) => todo!(),
            Transform::DefSymU16(s, _) => todo!(),
            Transform::DefSymU32(s, _) => todo!(),
            Transform::DefSymU64(s, _) => todo!(),
            Transform::DefSymAddress(s, _) => ctx.def_sym(
                SymbolKey::Address(
                    Self::to_addr(data, arch).ok_or(Error::TransformOutOfData(ctx.org))?,
                ),
                Symbol::new(s.clone(), SymbolKind::Label),
            ),
            Transform::Skip => (),
        }

        Ok(self.data_len(arch))
    }

    fn to_addr(data: &[u8], arch: &ArchDef) -> Option<Address> {
        if arch.endianess == Endianess::Little {
            Self::to_addr_le(data, arch)
        } else {
            Self::to_addr_be(data, arch)
        }
    }

    fn to_addr_be(data: &[u8], arch: &ArchDef) -> Option<Address> {
        Some(match arch.addr_size {
            1 => u8::from_be_bytes(data.try_into().ok()?) as Address,
            2 => u16::from_be_bytes(data.try_into().ok()?) as Address,
            4 => u32::from_be_bytes(data.try_into().ok()?) as Address,
            8 => u64::from_be_bytes(data.try_into().ok()?) as Address,
            _ => return None,
        })
    }

    fn to_addr_le(data: &[u8], arch: &ArchDef) -> Option<Address> {
        Some(match arch.addr_size {
            1 => u8::from_le_bytes(data.try_into().ok()?) as Address,
            2 => u16::from_le_bytes(data.try_into().ok()?) as Address,
            4 => u32::from_le_bytes(data.try_into().ok()?) as Address,
            8 => u64::from_le_bytes(data.try_into().ok()?) as Address,
            _ => return None,
        })
    }

    fn data_len(&self, arch: &ArchDef) -> usize {
        match self {
            Transform::AbsU8Hex(_) => 1,
            Transform::AbsU16Hex(_) => 2,
            Transform::AbsU32Hex(_) => 4,
            Transform::AbsU64Hex(_) => 8,
            Transform::String(_) => 0,
            Transform::DefSymU8(_, _) => 1,
            Transform::DefSymU16(_, _) => 2,
            Transform::DefSymU32(_, _) => 4,
            Transform::DefSymU64(_, _) => 8,
            Transform::DefSymAddress(_, _) => arch.addr_size,
            Transform::Skip => 0,
        }
    }

    fn offset(&self) -> usize {
        match self {
            Transform::AbsU8Hex(o) => *o,
            Transform::AbsU16Hex(o) => *o,
            Transform::AbsU32Hex(o) => *o,
            Transform::AbsU64Hex(o) => *o,
            Transform::String(_) => 0,
            Transform::DefSymU8(_, o) => *o,
            Transform::DefSymU16(_, o) => *o,
            Transform::DefSymU32(_, o) => *o,
            Transform::DefSymU64(_, o) => *o,
            Transform::DefSymAddress(_, o) => *o,
            Transform::Skip => 0,
        }
    }

    fn get_data(data: &[u8], offset: usize, len: usize) -> Option<&[u8]> {
        // TODO do we need this check or will data.get take care of that?
        if len == 0 {
            Some(&[])
        } else {
            data.get(offset..offset + len)
        }
    }
}

/// A matcher matches the pattern list and if *all* patterns match
/// it will apply the formatter
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Matcher {
    patterns: Vec<PatternList>,
    transforms: Vec<Transform>,
}

type MatcherList = Vec<Matcher>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, PartialEq, Eq)]
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
    // the context may provide additional matchers to apply
    // the context's matchers are executed before
    // the archdef's and will therefore override
    // the arch's default
    patterns: MatcherList,
    org: Address,
    syms: SymbolList,
}

impl Context {
    pub fn disas<T>(
        &mut self,
        f: impl DisasCallback<T>,
        data: &[u8],
        arch: &ArchDef,
        u: T,
    ) -> FdResult<()> {
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
    patterns: MatcherList,
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
        f: impl DisasCallback<T>,
        data: &[u8],
        ctx: &mut Context,
        u: T,
    ) -> FdResult<()> {
        Ok(())
    }
}
