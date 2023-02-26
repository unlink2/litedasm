use std::collections::BTreeMap;

use crate::core::dasm::arch::Archs;

use super::{a6502::InstructionMap, Arch, MatcherList, TransformMap};

use lazy_static::lazy_static;

lazy_static! {
    /// Built-in architecture for the 65c816 family
    pub static ref ARCH: Archs = Archs {archs: archs(), ..Default::default()};
}

pub(super) const ABSOLUTE24: &str = "absolute24";
pub(super) const DIRECT24: &str = "direct24"; // [DIRECT]
pub(super) const INDIRECT_Y24: &str = "indirect_y24"; // [DIRECT],y
pub(super) const LONG: &str = "long";
pub(super) const LONG_X: &str = "long_x";
pub(super) const RELATIVE16: &str = "relative16"; // long branches
pub(super) const SRC_DST: &str = "src_dst";
pub(super) const STACK_S: &str = "stack_s";
pub(super) const STACK_S_Y: &str = "stack_s_y";

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
