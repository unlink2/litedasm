use std::collections::BTreeMap;

use crate::core::dasm::arch::Archs;

use super::{a6502::InstructionMap, Arch, MatcherList, TransformMap};

use lazy_static::lazy_static;

lazy_static! {
    /// Built-in architecture for the 65c816 family
    pub static ref ARCH: Archs = Archs {archs: archs(), ..Default::default()};
}

pub(super) fn transforms() -> TransformMap {
    let mut map = super::a6502::transforms();
    map
}

pub(super) fn matchers_from(matchers: &mut MatcherList, instrs: InstructionMap) {
    for (k, modes) in instrs.iter() {}
    super::a65c02::matchers_from(matchers, instrs);
}

fn instruction_map() -> InstructionMap {
    InstructionMap::from([])
}

pub(super) fn patterns() -> MatcherList {
    let mut list = super::a65c02::patterns();
    matchers_from(&mut list, instruction_map());
    list
}

pub(super) fn archs() -> BTreeMap<String, Arch> {
    let mut map = BTreeMap::default();
    map.insert(
        "".into(),
        Arch {
            patterns: super::a6502::add_patterns_default(patterns()),
            transforms: transforms(),
            // we can unwrap this because the 6502 is guaranteed to have an empty
            // arch key!
            ..super::a6502::ARCH.archs.get("").unwrap().to_owned()
        },
    );
    map
}
