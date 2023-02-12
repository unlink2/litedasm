use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Address;

/// combination of value and size for a symbol
/// Using this makes it possible to define symbols with similar
/// names but different "data types"
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, PartialOrd, PartialEq, Ord, Eq, Copy, Clone)]
pub enum SymbolKey {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I32(i32),
    I64(i64),
    Address(Address),
    #[default]
    None,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Copy, Clone)]
pub enum SymbolKind {
    #[default]
    Const,
    Label,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Symbol {
    name: Option<String>,
    kind: SymbolKind,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct SymbolList {
    map: BTreeMap<SymbolKey, Symbol>,
}

impl SymbolList {
    pub fn def_symbol(&mut self, key: SymbolKey, sym: Symbol) -> Option<Symbol> {
        if key == SymbolKey::None {
            panic!("It is not possible to define a symbol with a key of None!");
        }
        self.map.insert(key, sym)
    }

    pub fn get_symbol(&self, key: SymbolKey) -> Option<&Symbol> {
        self.map.get(&key)
    }
}
