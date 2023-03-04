#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{Address, ValueType};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Copy, Clone, PartialEq, Eq, Debug)]
pub enum SymbolKind {
    #[default]
    Const,
    Label,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Copy, Clone, Debug)]
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

// TODO implement ord - sort by value and then sort the symbol list
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone, Debug)]
pub struct Symbol {
    #[cfg_attr(feature = "serde", serde(default))]
    pub name: String,
    #[cfg_attr(feature = "serde", serde(default))]
    pub kind: SymbolKind,
    #[cfg_attr(feature = "serde", serde(default))]
    pub scope: Scope,

    pub value: ValueType,
    pub len: usize,
}

impl Symbol {
    pub fn new(name: String, kind: SymbolKind, scope: Scope, value: ValueType, len: usize) -> Self {
        Self {
            name,
            kind,
            scope,
            value,
            len,
        }
    }

    pub fn is_match(&self, value: ValueType, address: Option<Address>) -> bool {
        let in_scope = if let Some(address) = address {
            self.scope.is_in_scope(address)
        } else {
            true
        };
        value >= self.value && value < self.value + self.len as ValueType && in_scope
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, Clone)]
pub struct SymbolList {
    #[cfg_attr(feature = "serde", serde(default))]
    map: Vec<Symbol>,
}

impl SymbolList {
    pub fn def_symbol(&mut self, sym: Symbol) {
        self.map.push(sym);
    }

    // get all symbols for a specific value
    pub fn get_symbols(&self, value: ValueType) -> Vec<Symbol> {
        self.map
            .iter()
            .filter(|x| x.is_match(value, None))
            .cloned()
            .collect()
    }

    pub fn get_first_symbol(&self, value: ValueType, address: Address) -> Option<&Symbol> {
        self.map.iter().find(|x| x.is_match(value, Some(address)))
    }

    // does any symbol in scope exist?
    pub fn has_symbols(&self, value: ValueType, address: Address) -> bool {
        self.map.iter().any(|x| x.is_match(value, Some(address)))
    }
}
