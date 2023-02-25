use std::collections::BTreeMap;

use crate::core::dasm::arch::Archs;

use super::Arch;

use lazy_static::lazy_static;

lazy_static! {
    /// Built-in architecture for the 6502 family
    pub static ref ARCH: Archs = Archs {archs: archs(), ..Default::default()};
}

fn archs() -> BTreeMap<String, Arch> {
    let mut map = BTreeMap::default();
    map.insert(
        "".into(),
        Arch {
            patterns: super::a6502::patterns(),
            transforms: super::a6502::transforms(),
            // we can unwrap this because the 6502 is guaranteed to have an empty
            // arch key!
            ..super::a6502::ARCH.archs.get("").unwrap().to_owned()
        },
    );
    map
}
