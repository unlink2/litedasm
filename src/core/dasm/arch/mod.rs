pub mod a6502;
pub mod a65c02;
pub mod a65c816;

use std::{collections::BTreeMap, fmt::Display};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::{Error, FdResult};

use super::{
    patch::Patch,
    symbols::{Scope, Symbol, SymbolKind, SymbolList},
    try_to_node, Address, DataType, ValueType, ValueTypeFmt,
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub enum NodeKind {
    Value(ValueType),
    Instruction,
    #[default]
    Static,
}

// the data that is passed to
// the callback for a decoded instruction
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Node {
    pub string: String,
    pub kind: NodeKind,
}

impl Node {
    pub fn new(string: String) -> Self {
        Self {
            string,
            ..Default::default()
        }
    }
}

impl From<&str> for Node {
    fn from(value: &str) -> Self {
        Node::new(value.into())
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.string)
    }
}

/// This callback is called for every matched pattern with the final
/// transformed result. Each context may also pass along a user-data field <T>
/// which can be used to make the callback work
pub trait DisasCallback = FnMut(&Node, &[u8], &Arch, &mut Context) -> FdResult<()>;

pub fn default_callback(
    node: &Node,
    _raw: &[u8],
    _arch: &Arch,
    _ctx: &mut Context,
) -> FdResult<()> {
    print!("{}", node.string);
    Ok(())
}

/// A match pattern
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone, PartialEq)]
pub enum Pattern {
    Exact(u8),
    And(u8),
    List(PatternList),
    Any,
    Always,
    // Match an address range from 0..1
    Address(Address, Address),
    // check if a flag has a certain value
    Flag(String, Option<String>),
    #[default]
    Never,
}

impl Pattern {
    pub fn is_match(&self, _arch: &Arch, ctx: &mut Context, byte: u8) -> bool {
        match self {
            Self::Exact(b) => *b == byte,
            Self::And(b) => *b & byte != 0,
            Self::List(l) => l.iter().fold(true, |i, p| i & p.is_match(_arch, ctx, byte)),
            Self::Address(start, end) => ctx.address() >= *start && ctx.address() < *end,
            Self::Any => true,
            Self::Always => true,
            Self::Never => false,
            Self::Flag(key, value) => ctx.get_flag(key) == value.as_ref(),
        }
    }

    pub fn always(&self) -> bool {
        *self == Self::Always
    }
}

type PatternList = Vec<Pattern>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct DefSym {
    #[cfg_attr(feature = "serde", serde(default))]
    name: String,
    #[cfg_attr(feature = "serde", serde(default))]
    offset: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    scope: Scope,
    #[cfg_attr(feature = "serde", serde(default))]
    data_type: DataType,
    #[cfg_attr(feature = "serde", serde(default))]
    symbol_kind: SymbolKind,
    #[cfg_attr(feature = "serde", serde(default))]
    rel: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    len: usize,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct ValOut {
    #[cfg_attr(feature = "serde", serde(default))]
    offset: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    fmt: ValueTypeFmt,
    #[cfg_attr(feature = "serde", serde(default))]
    data_type: DataType,

    #[cfg_attr(feature = "serde", serde(default))]
    rel: bool,

    // automatically define a symbol for this
    // value of no symbol does yet exist at the requested value
    #[cfg_attr(feature = "serde", serde(default))]
    auto_def_sym: bool,
}

/// A formatter takes an input &[u8] and applies a transform to the data
/// then it outputs its contents to anything with a dyn Write trait  
/// TODO implement a prefix and postfix system that can change its output depending on
///      if the data in abs or rel is a label or not
/// TODO implement transforms that can switch org and architecture on the fly
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub enum Transform {
    /// The absolute formatters take in the input array,
    /// and attempt to read its absolute value at a given offset
    /// from the array
    /// AbsXX takes the offset and radix
    Val(ValOut),
    // output label at current address
    Label,
    /// A static node that can be applied at any point
    /// this node has no attached size
    Static(Node),
    /// A node that outputs the matcher's name
    MatcherName,
    /// Defsym takes a string and defines a clear name
    /// for the given value at the requested offset
    DefSym(DefSym),
    DefSymAddress(DefSym),
    // consume n bytes
    Consume(usize),
    // Outputs the current address with a prefix of n 0s
    Address(usize),
    OffsetAddress(i64),
    SetAddress(Address),
    SetFlag(String, String),
    UnsetFlag(String),
    ChangeArch(String),
    #[default]
    Skip,
}

impl Transform {
    pub fn new_line() -> Self {
        Self::Static(Node::new("\n".into()))
    }

    pub fn tab() -> Self {
        Self::Static(Node::new("\t".into()))
    }

    pub fn space(n: usize) -> Self {
        Self::Static(Node::new(" ".repeat(n)))
    }

    pub fn apply(
        &self,
        f: &mut dyn DisasCallback,
        data: &[u8],
        arch: &Arch,
        ctx: &mut Context,
        matcher_name: &Node,
    ) -> FdResult<usize> {
        // get all data, if no data is available just return with an error
        // since a transform should *never* be out of data
        // assuming the pattern is defined correctly!
        let data = Self::get_data(data, self.offset(), self.read_len(arch.addr_type))
            .ok_or(Error::TransformOutOfData(ctx.org))?;

        let dt = self.data_type(arch.addr_type);

        if ctx.analyze {
            self.analyze(f, data, arch, ctx, matcher_name)?;
        } else {
            self.no_analyze(f, data, arch, ctx, matcher_name, dt)?;
        }
        self.analyze_and_no_analyze(f, data, arch, ctx, matcher_name, dt)?;

        Ok(self.data_len())
    }

    fn no_analyze(
        &self,
        f: &mut dyn DisasCallback,
        data: &[u8],
        arch: &Arch,
        ctx: &mut Context,
        matcher_name: &Node,
        _dt: DataType,
    ) -> FdResult<()> {
        match self {
            Transform::Val(ao) => self.output_value(f, data, arch, ctx, ao)?,
            Transform::Label => self.output_label(f, data, arch, ctx)?,
            Transform::Static(s) => f(s, data, arch, ctx)?,
            Transform::MatcherName => f(matcher_name, data, arch, ctx)?,
            Transform::Address(width) => f(
                &Node::new(format!("{:0width$x}", ctx.address())),
                data,
                arch,
                ctx,
            )?,
            _ => {}
        }
        Ok(())
    }

    fn analyze_and_no_analyze(
        &self,
        _f: &mut dyn DisasCallback,
        _data: &[u8],
        _arch: &Arch,
        ctx: &mut Context,
        _matcher_name: &Node,
        _dt: DataType,
    ) -> FdResult<()> {
        match self {
            Transform::Skip => (),
            Transform::Consume(_) => {}
            Transform::OffsetAddress(change) => {
                ctx.offset = ctx.offset.wrapping_add(*change as Address)
            }
            Transform::SetAddress(change) => {
                ctx.org = *change;
                ctx.offset = 0;
            }
            Transform::SetFlag(key, value) => {
                ctx.def_flag(key, value);
            }
            Transform::UnsetFlag(key) => {
                ctx.undef_flag(key);
            }
            Transform::ChangeArch(val) => ctx.arch_key = val.to_owned(),
            _ => {}
        }
        Ok(())
    }

    fn analyze(
        &self,
        _f: &mut dyn DisasCallback,
        data: &[u8],
        arch: &Arch,
        ctx: &mut Context,
        _matcher_name: &Node,
    ) -> FdResult<()> {
        match self {
            Transform::DefSym(ds) => ctx.def_symbol(Symbol::new(
                ds.name.clone(),
                ds.symbol_kind,
                ds.scope,
                Self::to_value(data, arch)?,
                ds.len,
            )),
            Transform::DefSymAddress(ds) => ctx.def_symbol(Symbol::new(
                ds.name.clone(),
                ds.symbol_kind,
                ds.scope,
                Self::to_addr(data, arch)?,
                ds.len,
            )),
            _ => {}
        }
        Ok(())
    }

    fn output_label(
        &self,
        f: &mut dyn DisasCallback,
        data: &[u8],
        arch: &Arch,
        ctx: &mut Context,
    ) -> FdResult<()> {
        let labels = ctx.syms.get_symbols(ctx.address() as ValueType);
        let mut result = "".to_owned();
        for label in labels {
            if label.scope.is_in_scope(ctx.address()) && label.kind == SymbolKind::Label {
                result.push_str(&format!("{}:\n", &label.name));
            }
        }
        f(&Node::new(result), data, arch, ctx)
    }

    fn output_value(
        &self,
        f: &mut dyn DisasCallback,
        data: &[u8],
        arch: &Arch,
        ctx: &mut Context,
        ao: &ValOut,
    ) -> FdResult<()> {
        let value = Self::to_value(data, arch)?;

        let sym_val = if ao.rel {
            let addr = (ctx.address() as ValueType).wrapping_add(value);
            addr as ValueType & ao.data_type.mask()
        } else {
            value
        };

        if let Some(sym) = ctx.get_first_symbol(sym_val) {
            if !ctx.analyze {
                let sym_name = if sym.value == sym_val {
                    sym.name.to_owned()
                } else {
                    // represent the actual difference between the current
                    // value and the symbol's value here
                    format!("{}+{}", sym.name, sym_val - sym.value)
                };
                f(&Node::new(sym_name), data, arch, ctx)?
            }
        } else if !ctx.analyze {
            f(&try_to_node(value, ao.fmt, arch)?, data, arch, ctx)?
        } else if ao.auto_def_sym {
            let name = format!("auto_{}", ctx.address());
            ctx.def_symbol(Symbol::new(
                name,
                SymbolKind::Label,
                Scope::Global,
                value,
                1,
            ));
        }

        Ok(())
    }

    fn to_addr(data: &[u8], arch: &Arch) -> FdResult<ValueType> {
        arch.endianess
            .transform(data)
            .ok_or(Error::TransformOutOfData(0))
    }

    fn to_value(data: &[u8], arch: &Arch) -> FdResult<ValueType> {
        let data = arch.endianess.pad(data, std::mem::size_of::<ValueType>());
        arch.endianess
            .transform(&data)
            .ok_or(Error::TransformOutOfData(0))
    }

    fn data_type(&self, addr_type: DataType) -> DataType {
        match self {
            Transform::Val(ao) => ao.data_type,
            Transform::DefSym(ds) => ds.data_type,
            Transform::DefSymAddress(_) => addr_type,
            _ => DataType::None,
        }
    }

    // returns amount of bytes that should be read, but *not* consumed
    fn read_len(&self, addr_type: DataType) -> usize {
        match self {
            Transform::Val(_) => self.data_len(),
            _ => self.data_type(addr_type).data_len(),
        }
    }

    fn offset(&self) -> usize {
        match self {
            Transform::Val(ao) => ao.offset,
            Transform::DefSym(ds) => ds.offset,
            Transform::DefSymAddress(ds) => ds.offset,
            _ => 0,
        }
    }

    // returns amount of bytes that should be consumed
    fn data_len(&self) -> usize {
        match self {
            Transform::Val(dt) => dt.data_type.data_len(),
            Transform::Consume(skip) => *skip,
            _ => 0,
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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct PatternAt {
    offset: usize,
    pattern: Pattern,
}

impl PatternAt {
    pub fn new(pattern: Pattern, offset: usize) -> Self {
        Self { pattern, offset }
    }
}

/// A matcher matches the pattern list and if *all* patterns match
/// it will apply the formatter
/// the transforms are referened by the transform string
/// which can be applied when the patterns match
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Matcher {
    // a list of patterns that have to match for this matcher to be
    // a full match
    patterns: Vec<PatternAt>,
    // the name of the transform to apply in case of a match
    transforms: String,
    // the name of this matcher
    name: Node,
}

impl Matcher {
    /// check if this matcher matches the pattern specified
    pub fn is_match(&self, arch: &Arch, ctx: &mut Context, data: &[u8]) -> bool {
        for pa in self.patterns.iter() {
            if pa.pattern.always() {
                return true;
            }
            if let Some(byte) = data.get(pa.offset) {
                if !pa.pattern.is_match(arch, ctx, *byte) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    /// apply the apropriate transform for this matcher
    pub fn transform(
        &self,
        mut f: impl DisasCallback,
        data: &[u8],
        arch: &Arch,
        ctx: &mut Context,
    ) -> FdResult<usize> {
        if let Some(tl) = arch.get_transform(&self.transforms) {
            let mut total = 0;
            total = self.apply(&mut f, data, arch, ctx, total, tl)?;
            Ok(total)
        } else {
            Err(Error::TransformNotFound(self.transforms.clone()))
        }
    }

    fn apply(
        &self,
        f: &mut dyn DisasCallback,
        data: &[u8],
        arch: &Arch,
        ctx: &mut Context,
        total: usize,
        tl: &TransformList,
    ) -> FdResult<usize> {
        let mut total = total;
        for t in tl.iter() {
            let read = t.apply(f, &data[total..], arch, ctx, &self.name)?;
            total += read;
            ctx.offset += read as Address;
        }
        Ok(total)
    }
}

type MatcherList = Vec<Matcher>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, PartialEq, Eq, Copy, Clone)]
pub enum Endianess {
    // controversial default
    #[default]
    Little,
    Big,
}

impl Endianess {
    pub fn transform(&self, data: &[u8]) -> Option<ValueType> {
        if self == &Self::Little {
            Self::transform_le(data)
        } else {
            Self::transform_be(data)
        }
    }

    // if there is a mismatch between data len and len
    // pad and return as a new vec
    // FIXME maybe use stack allocated vec here in the future
    pub fn pad(&self, data: &[u8], len: usize) -> Vec<u8> {
        if data.len() >= len {
            return data.to_vec();
        }
        let diff = len - data.len();
        match self {
            Endianess::Big => {
                let mut v = vec![0; diff];
                v.append(&mut data.to_vec());
                v
            }
            Endianess::Little => {
                let mut v = data.to_vec();
                v.append(&mut vec![0; diff]);
                v
            }
        }
    }

    fn transform_le(data: &[u8]) -> Option<ValueType> {
        Some(ValueType::from_le_bytes(data.try_into().ok()?))
    }

    fn transform_be(data: &[u8]) -> Option<ValueType> {
        Some(ValueType::from_be_bytes(data.try_into().ok()?))
    }
}

/// The context describes the runtime information of a single parser operation
/// it contains the current address as well as a list of known symbols
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Context {
    // which architecture to use
    #[cfg_attr(feature = "serde", serde(default))]
    pub arch_key: String,
    // a list of flags that can be set or unset using transforms
    #[cfg_attr(feature = "serde", serde(default))]
    pub flags: BTreeMap<String, String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub org: Address,
    #[cfg_attr(feature = "serde", serde(default))]
    pub offset: Address,
    #[cfg_attr(feature = "serde", serde(default))]
    pub start_read: usize,
    #[cfg_attr(feature = "serde", serde(default))]
    pub end_read: Option<usize>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub syms: SymbolList,
    #[cfg_attr(feature = "serde", serde(default))]
    pub analyze: bool,

    // a file can optionally be patched from data and
    // from a patch file
    #[cfg_attr(feature = "serde", serde(default))]
    pub patches: Vec<Patch>,
}

impl Context {
    pub fn new(org: Address, syms: SymbolList) -> Self {
        Self {
            arch_key: "".into(),
            flags: Default::default(),
            org,
            syms,
            offset: 0,
            analyze: false,
            start_read: 0,
            end_read: None,
            patches: Default::default(),
        }
    }

    pub fn patch(&self, data: &[u8]) -> FdResult<Vec<u8>> {
        let mut data = data.to_vec();
        self.patches.iter().try_for_each(|x| x.apply(&mut data))?;
        Ok(data)
    }

    pub fn restart(&mut self) {
        self.offset = 0;
    }

    pub fn address(&self) -> Address {
        self.org + self.offset
    }

    pub fn def_symbol(&mut self, sym: Symbol) {
        self.syms.def_symbol(sym);
    }

    pub fn get_first_symbol(&self, value: ValueType) -> Option<&Symbol> {
        self.syms.get_first_symbol(value, self.address())
    }

    pub fn def_flag(&mut self, flag: &str, value: &str) {
        self.flags.insert(flag.into(), value.into());
    }

    pub fn undef_flag(&mut self, flag: &str) {
        self.flags.remove(flag);
    }

    pub fn get_flag(&self, flag: &str) -> Option<&String> {
        self.flags.get(flag)
    }

    pub fn set_start(&mut self, addr: usize) {
        self.start_read = addr.wrapping_sub(self.org as usize);
    }

    pub fn set_end(&mut self, addr: Option<usize>) {
        if let Some(addr) = addr {
            self.end_read = Some(addr.wrapping_sub(self.org as usize));
        } else {
            self.end_read = addr;
        }
    }
}

pub type TransformMap = BTreeMap<String, TransformList>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Arch {
    // a list of all possible patterns this architecture may match against
    #[cfg_attr(feature = "serde", serde(default))]
    patterns: MatcherList,
    // a list of named transforms. they can be referenced by the
    // context based on a match result
    #[cfg_attr(feature = "serde", serde(default))]
    transforms: TransformMap,

    // transforms that are applied before and after every match
    #[cfg_attr(feature = "serde", serde(default))]
    pre_patterns: MatcherList,
    #[cfg_attr(feature = "serde", serde(default))]
    post_patterns: MatcherList,

    #[cfg_attr(feature = "serde", serde(default))]
    endianess: Endianess,
    // size of address in bytes for the given architecture
    #[cfg_attr(feature = "serde", serde(default))]
    addr_type: DataType,

    /// This map can be referenced during
    /// some phaes of disassembly
    /// e.g. formatting a value will take fmt_<fmt>_pre and fmt_<fmt>_post
    /// to allow for prefixing and postfixing of
    /// the output value e.g.
    /// fmt_hex_pre = $ will prefix any hex number with a $ ($4B)
    /// Valid format type keys:
    ///     fmt_hex_pre, fmt_hex_post, fmt_HEX_pre, fmt_HEX_post,
    ///     fmt_dec_pre, fmt_dec_post, fmt_oct_pre, fmt_oct_post,
    ///     fmt_bin_pre, fmt_bin_post
    #[cfg_attr(feature = "serde", serde(default))]
    pub node_map: BTreeMap<String, Node>,
}

impl Arch {
    /// Match a pattern
    /// All transforms that match a pattern are applied
    /// until the first transform is hit that consumes actual data
    /// This means that transforms that do not require data can be used as
    /// a conditional prefix e.g. to insert assembler directives
    /// at a certain address
    /// and after those are matched regular instructions can be
    /// parsed
    fn match_patterns(
        &self,
        f: &mut dyn DisasCallback,
        data: &[u8],
        ctx: &mut Context,
    ) -> FdResult<usize> {
        for pattern in self.patterns.iter() {
            if pattern.is_match(self, ctx, data) {
                let mut res = self.match_additional_patterns(f, data, ctx, &self.pre_patterns)?;
                res += pattern.transform(&mut *f, &data[res..], self, ctx)?;
                res += self.match_additional_patterns(f, &data[res..], ctx, &self.post_patterns)?;

                return Ok(res);
            }
        }
        Err(Error::NoMatch)
    }

    fn match_additional_patterns(
        &self,
        f: &mut dyn DisasCallback,
        data: &[u8],
        ctx: &mut Context,
        patterns: &MatcherList,
    ) -> FdResult<usize> {
        for pattern in patterns.iter() {
            if pattern.is_match(self, ctx, data) {
                let res = pattern.transform(&mut *f, data, self, ctx)?;

                return Ok(res);
            }
        }
        Ok(0)
    }

    pub fn get_transform(&self, name: &str) -> Option<&TransformList> {
        self.transforms.get(name)
    }
}

// a collection of many architectures
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Archs {
    // a list of archs the context may select
    archs: BTreeMap<String, Arch>,

    // initial org for context
    #[cfg_attr(feature = "serde", serde(default))]
    org: Address,
    // inital symbol list
    #[cfg_attr(feature = "serde", serde(default))]
    syms: SymbolList,
}

impl Archs {
    /// start disasssembly
    /// This will write all result strings to the f callback,
    /// and it will modify the current context
    /// This call creates a default context and returns it
    pub fn disas(&self, f: impl DisasCallback, data: &[u8]) -> FdResult<Context> {
        let mut ctx = Context::new(self.org, self.syms.clone());
        self.disas_ctx(f, data, &mut ctx)?;
        Ok(ctx)
    }

    /// Call the disas function with an existing context
    pub fn disas_ctx(
        &self,
        mut f: impl DisasCallback,
        data: &[u8],
        ctx: &mut Context,
    ) -> FdResult<()> {
        let end_read = ctx.end_read.unwrap_or(data.len());
        let start_read = ctx.start_read;
        let data = &data[start_read..end_read];

        let mut total = 0;
        // loop until total data processed is out of range
        // or an error occured
        while total < data.len() {
            let arch = self
                .archs
                .get(&ctx.arch_key)
                .ok_or_else(|| Error::ArchNotFound(ctx.arch_key.clone()))?;
            total += arch.match_patterns(&mut f, &data[total..], ctx)?;
        }
        Ok(())
    }
}
