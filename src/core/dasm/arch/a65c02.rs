use std::collections::BTreeMap;

use crate::core::dasm::{arch::Archs, DataType, ValueTypeFmt};

use super::{
    a6502::{matcher2, InstructionMap, ModeMap},
    Arch, MatcherList, Node, Transform, TransformMap, ValOut,
};

use lazy_static::lazy_static;

lazy_static! {
    /// Built-in architecture for the 6502 family
    pub static ref ARCH: Archs = Archs {archs: archs(), ..Default::default()};
}

const INDIRECT: &str = "indirect";

fn transform_indirect(map: &mut TransformMap) {
    map.insert(
        INDIRECT.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" (".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
                ..Default::default()
            }),
            Transform::Static(Node::new(")".into())),
        ],
    );
}

pub(super) fn transforms() -> TransformMap {
    let mut map = super::a6502::transforms();
    transform_indirect(&mut map);
    map
}

fn indirect_instruction_map(name: &'static str, op: u8) -> (&'static str, ModeMap) {
    (name, ModeMap::from([(INDIRECT, op)]))
}

fn instruction_map() -> InstructionMap {
    InstructionMap::from([
        indirect_instruction_map("ora", 0x12),
        indirect_instruction_map("and", 0x32),
        indirect_instruction_map("eor", 0x52),
        indirect_instruction_map("adc", 0x72),
        indirect_instruction_map("sta", 0x92),
        indirect_instruction_map("lda", 0xB2),
        indirect_instruction_map("cmp", 0xD2),
        indirect_instruction_map("sbc", 0xF2),
    ])
}

fn matcher_indirect(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, INDIRECT);
}

pub(super) fn matchers_from(matchers: &mut MatcherList, instrs: InstructionMap) {
    for (k, modes) in instrs.iter() {
        if let Some(op) = modes.get(INDIRECT) {
            matcher_indirect(matchers, *op, k);
        }
    }
    super::a6502::matchers_from(matchers, instrs);
}

pub(super) fn patterns() -> MatcherList {
    let mut list = super::a6502::patterns();
    matchers_from(&mut list, instruction_map());
    list
}

fn archs() -> BTreeMap<String, Arch> {
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
