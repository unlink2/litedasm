use std::collections::BTreeMap;

use super::Address;

pub struct Symbol {
    address: Address,
    name: Option<String>,
}

pub struct SymbolList {
    map: BTreeMap<Address, Symbol>,
}

impl SymbolList {
    pub fn def_symbol(&mut self, sym: Symbol) {}

    pub fn get_symbol(&self) -> &Symbol {
        todo!()
    }
}
