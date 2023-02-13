use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Error, FdResult};

use super::{
    symbols::{Symbol, SymbolKey, SymbolKind, SymbolList},
    Address, DataType, ValueType,
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
    // Match an address range from 0..1
    Address(Address, Address),
    #[default]
    Never,
}

impl Pattern {
    pub fn is_match(&self, arch: &ArchDef, ctx: &mut Context, byte: u8) -> bool {
        match self {
            Self::Exact(b) => *b == byte,
            Self::And(b) => *b & byte != 0,
            Self::List(l) => l.iter().fold(true, |i, p| i & p.is_match(arch, ctx, byte)),
            Self::Address(start, end) => ctx.address() >= *start && ctx.address() < *end,
            Self::Any => true,
            Self::Never => false,
        }
    }
}

type PatternList = Vec<Pattern>;

impl ValueType {}

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
        let data = Self::get_data(
            data,
            self.offset(),
            self.data_type(arch.addr_type).data_len(),
        )
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
                Self::to_addr(data, arch).ok_or(Error::TransformOutOfData(ctx.org))?,
                Symbol::new(s.clone(), SymbolKind::Label),
            ),
            Transform::Skip => (),
        }

        Ok(self.data_type(arch.addr_type).data_len())
    }

    fn to_addr(data: &[u8], arch: &ArchDef) -> Option<ValueType> {
        arch.endianess.transform(data, arch.addr_type)
    }

    fn data_type(&self, addr_type: DataType) -> DataType {
        match self {
            Transform::AbsU8Hex(_) => DataType::U8,
            Transform::AbsU16Hex(_) => DataType::U16,
            Transform::AbsU32Hex(_) => DataType::U32,
            Transform::AbsU64Hex(_) => DataType::U64,
            Transform::String(_) => DataType::None,
            Transform::DefSymU8(_, _) => DataType::U8,
            Transform::DefSymU16(_, _) => DataType::U16,
            Transform::DefSymU32(_, _) => DataType::U32,
            Transform::DefSymU64(_, _) => DataType::U64,
            Transform::DefSymAddress(_, _) => addr_type,
            Transform::Skip => DataType::None,
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

pub type TransformList = Vec<Transform>;

/// A matcher matches the pattern list and if *all* patterns match
/// it will apply the formatter
/// the transforms are referened by the transform string
/// which can be applied when the patterns match
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Matcher {
    // a list of patterns that have to match for this matcher to be
    // a full match
    patterns: Vec<PatternList>,
    // the name of the transform to apply in case of a match
    transforms: String,
}

impl Matcher {
    /// check if this matcher matches the pattern specified
    pub fn is_match(&self, arch: &ArchDef, ctx: &mut Context, data: &[u8]) -> bool {
        todo!()
    }

    /// apply the apropriate transform for this matcher
    pub fn transform<T>(
        &self,
        mut f: impl DisasCallback<T>,
        data: &[u8],
        arch: &ArchDef,
        ctx: &mut Context,
        u: T,
    ) -> FdResult<usize> {
        todo!()
    }
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

impl Endianess {
    pub fn transform(&self, data: &[u8], dt: DataType) -> Option<ValueType> {
        if self == &Self::Little {
            Self::transform_le(data, dt)
        } else {
            Self::transform_be(data, dt)
        }
    }

    fn transform_le(data: &[u8], dt: DataType) -> Option<ValueType> {
        Some(match dt {
            DataType::U8 => ValueType::U8(u8::from_le_bytes(data.try_into().ok()?)),
            DataType::U16 => ValueType::U16(u16::from_le_bytes(data.try_into().ok()?)),
            DataType::U32 => ValueType::U32(u32::from_le_bytes(data.try_into().ok()?)),
            DataType::U64 => ValueType::U64(u64::from_le_bytes(data.try_into().ok()?)),
            DataType::None => ValueType::None,
            _ => todo!(),
        })
    }

    fn transform_be(data: &[u8], dt: DataType) -> Option<ValueType> {
        Some(match dt {
            DataType::U8 => ValueType::U8(u8::from_be_bytes(data.try_into().ok()?)),
            DataType::U16 => ValueType::U16(u16::from_be_bytes(data.try_into().ok()?)),
            DataType::U32 => ValueType::U32(u32::from_be_bytes(data.try_into().ok()?)),
            DataType::U64 => ValueType::U64(u64::from_be_bytes(data.try_into().ok()?)),
            DataType::None => ValueType::None,
            _ => todo!(),
        })
    }
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
    // TODO context should also be able to overwrite patterns at certain
    // addresses with another transform e.g. to interprete the data as bytes
    // instead of instructions
    // override named transforms from the architecture
    transforms: BTreeMap<String, TransformList>,

    org: Address,
    offset: Address,
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
        // TODO match both pattern lists here
        todo!()
    }

    fn match_patterns<'a, T>(
        &mut self,
        f: impl DisasCallback<T>,
        data: &'a [u8],
        arch: &ArchDef,
        u: T,
        patterns: &MatcherList,
    ) -> FdResult<&'a [u8]> {
        for pattern in patterns.iter() {
            if pattern.is_match(arch, self, data) {
                let res = pattern.transform(f, data, arch, self, u)?;
                return Ok(&data[res..]);
            }
        }
        Err(Error::NoMatch)
    }

    pub fn address(&self) -> Address {
        self.org + self.offset
    }

    pub fn def_sym(&mut self, key: SymbolKey, sym: Symbol) {
        self.syms.def_symbol(key, sym);
    }

    pub fn get_sym(&self, key: SymbolKey) -> Option<&Symbol> {
        self.syms.get_symbol(key)
    }

    // obtain the transform list for all possible matches
    pub fn get_transform(&self, name: &str) -> Option<&TransformList> {
        todo!()
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct ArchDef {
    patterns: MatcherList,
    // a list of named transforms. they can be referenced by the
    // context based on a match result
    transforms: BTreeMap<String, TransformList>,

    endianess: Endianess,
    // size of address in bytes for the given architecture
    addr_type: DataType,
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
        ctx.disas(f, data, self, u)
    }

    pub fn get_transform(&self, name: &str) -> Option<&TransformList> {
        todo!()
    }
}
