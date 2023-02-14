use std::collections::BTreeMap;

use super::{Arch, Matcher, MatcherList, Pattern, TransformList};
use lazy_static::lazy_static;

/// Built-in architecture for the 6502 family
lazy_static! {
    pub static ref ARCH: Arch = Arch {
        patterns: patterns(),
        transforms: transforms(),
        ..Arch::default()
    };
}

fn transforms() -> BTreeMap<String, TransformList> {
    let mut map = BTreeMap::default();

    map
}

fn patterns() -> MatcherList {
    vec![Matcher {
        patterns: vec![(0, Pattern::Any)],
        transforms: "define_byte".into(),
    }]
}
