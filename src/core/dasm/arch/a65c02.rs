use std::collections::BTreeMap;

use crate::core::dasm::{arch::Archs, DataType, ValueTypeFmt};

use super::{
    a6502::{
        implied_instruction_map, matcher2, matcher3, relative_instruction_map, InstructionMap,
        ModeMap, ABSOLUTE, ABSOLUTE_X, ACCUMULATOR, IMMEDIATE, ZP, ZP_X,
    },
    Arch, MatcherList, Node, Transform, TransformMap, ValOut,
};

use lazy_static::lazy_static;

lazy_static! {
    /// Built-in architecture for the 6502 family
    pub static ref ARCH: Archs = Archs {archs: archs(), ..Default::default()};
}

const INDIRECT: &str = "indirect";
const ABS_INDIRECT_X: &str = "abs_indirect_x";

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

fn transform_absolute_indirect_x(map: &mut TransformMap) {
    map.insert(
        ABS_INDIRECT_X.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" (".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(4),
                data_type: DataType::U16,
                ..Default::default()
            }),
            Transform::Static(Node::new(", x)".into())),
        ],
    );
}

pub(super) fn transforms() -> TransformMap {
    let mut map = super::a6502::transforms();
    transform_indirect(&mut map);
    transform_absolute_indirect_x(&mut map);
    map
}

fn indirect_instruction_map(name: &'static str, op: u8) -> (&'static str, ModeMap) {
    (name, ModeMap::from([(INDIRECT, op)]))
}

fn test_instruction_map(name: &'static str, zp: u8, abs: u8) -> (&'static str, ModeMap) {
    (name, ModeMap::from([(ZP, zp), (ABSOLUTE, abs)]))
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
        (
            "bit",
            ModeMap::from([(IMMEDIATE, 0x89), (ZP_X, 0x34), (ABSOLUTE_X, 0x3C)]),
        ),
        ("dec", ModeMap::from([(ACCUMULATOR, 0x3A)])),
        ("inc", ModeMap::from([(ACCUMULATOR, 0x1A)])),
        ("jmp", ModeMap::from([(ABS_INDIRECT_X, 0x7C)])),
        relative_instruction_map("bra", 0x80),
        implied_instruction_map("phx", 0xDA),
        implied_instruction_map("phy", 0x5A),
        implied_instruction_map("plx", 0xFA),
        implied_instruction_map("plx", 0xFA),
        implied_instruction_map("ply", 0x7A),
        (
            "stz",
            ModeMap::from([
                (ZP, 0x64),
                (ZP_X, 0x74),
                (ABSOLUTE, 0x9C),
                (ABSOLUTE_X, 0x9E),
            ]),
        ),
        test_instruction_map("trb", 0x14, 0x1C),
        test_instruction_map("tsb", 0x04, 0x0C),
    ])
}

fn matcher_indirect(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, INDIRECT);
}

fn matcher_absolute_indirect_x(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, ABS_INDIRECT_X);
}

pub(super) fn matchers_from(matchers: &mut MatcherList, instrs: InstructionMap) {
    for (k, modes) in instrs.iter() {
        if let Some(op) = modes.get(INDIRECT) {
            matcher_indirect(matchers, *op, k);
        }
        if let Some(op) = modes.get(ABS_INDIRECT_X) {
            matcher_absolute_indirect_x(matchers, *op, k);
        }
    }
    super::a6502::matchers_from(matchers, instrs);
}

pub(super) fn patterns() -> MatcherList {
    let mut list = super::a6502::patterns();
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
