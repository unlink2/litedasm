use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{Address, ValueType};

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
#[derive(Default, Copy, Clone)]
pub enum Scope {
    #[default]
    Global,
    Range(Address, Address),
}

impl Scope {
    pub fn is_in_scope(&self, address: Address) -> bool {
        match self {
            Self::Global => true,
            Self::Range(start, end) => address >= *start && address < *end,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct Symbol {
    #[cfg_attr(feature = "serde", serde(default))]
    name: String,
    #[cfg_attr(feature = "serde", serde(default))]
    kind: SymbolKind,
    #[cfg_attr(feature = "serde", serde(default))]
    scope: Scope,
}

impl Symbol {
    pub fn new(name: String, kind: SymbolKind, scope: Scope) -> Self {
        Self { name, kind, scope }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct SymbolList {
    #[cfg_attr(feature = "serde", serde(default))]
    map: BTreeMap<SymbolKey, Vec<Symbol>>,
}

impl SymbolList {
    pub fn def_symbol(&mut self, key: SymbolKey, sym: Symbol) {
        if key == SymbolKey::None {
            panic!("It is not possible to define a symbol with a key of None!");
        }
        if let Some(v) = self.map.get_mut(&key) {
            v.push(sym);
        } else {
            self.map.insert(key, vec![sym]);
        }
    }

    pub fn get_symbol(&self, key: SymbolKey, address: Address) -> Option<&Symbol> {
        self.map
            .get(&key)?
            .iter()
            .find(|x| x.scope.is_in_scope(address))
    }
}
