use std::collections::BTreeMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Address;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct Symbol {
    address: Address,
    name: Option<String>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default)]
pub struct SymbolList {
    map: BTreeMap<Address, Symbol>,
}

impl SymbolList {
    pub fn def_symbol(&mut self, sym: Symbol) {}

    pub fn get_symbol(&self) -> &Symbol {
        todo!()
    }
}
