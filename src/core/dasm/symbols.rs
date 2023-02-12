use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::ValueType;

/// combination of value and size for a symbol
/// Using this makes it possible to define symbols with similar
/// names but different "data types"
pub type SymbolKey = ValueType;

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
    name: String,
    kind: SymbolKind,
}

impl Symbol {
    pub fn new(name: String, kind: SymbolKind) -> Self {
        Self { name, kind }
    }
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
