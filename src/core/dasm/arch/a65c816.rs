use std::collections::BTreeMap;

use crate::core::dasm::{arch::Archs, DataType, ValueTypeFmt};

use super::{
    a6502::{
        implied_instruction_map, matcher2, matcher3, InstructionMap, ModeMap, ABSOLUTE, IMMEDIATE,
        IMMEDIATE16,
    },
    Arch, MatcherList, Node, Transform, TransformMap, ValOut,
};

use lazy_static::lazy_static;

lazy_static! {
    /// Built-in architecture for the 65c816 family
    pub static ref ARCH: Archs = Archs {archs: archs(), ..Default::default()};
}

pub(super) const DIRECT24: &str = "direct24"; // [DIRECT]
pub(super) const INDIRECT_Y24: &str = "indirect_y24"; // [DIRECT],y
pub(super) const LONG: &str = "long";
pub(super) const LONG_X: &str = "long_x";
pub(super) const RELATIVE16: &str = "relative16"; // long branches
pub(super) const STACK_S: &str = "stack_s";
pub(super) const STACK_S_Y: &str = "stack_s_y";
pub(super) const MOVE: &str = "move";
pub(super) const JUMP_LONG_INDIRECT: &str = "jump_long_indirect";
pub(super) const JSR_INDIRECT_X: &str = "jsr_indirect_x";

fn transform_stack_s(map: &mut TransformMap) {
    map.insert(
        STACK_S.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" ".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
                ..Default::default()
            }),
            Transform::Static(Node::new(", s".into())),
        ],
    );
}

fn transform_stack_s_y(map: &mut TransformMap) {
    map.insert(
        STACK_S_Y.into(),
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
            Transform::Static(Node::new(", s), y".into())),
        ],
    );
}

fn transform_direct24(map: &mut TransformMap) {
    map.insert(
        DIRECT24.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" [".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
                ..Default::default()
            }),
            Transform::Static(Node::new("]".into())),
        ],
    );
}

fn transform_jump_long_indirect(map: &mut TransformMap) {
    map.insert(
        JUMP_LONG_INDIRECT.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" [".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(4),
                data_type: DataType::U16,
                ..Default::default()
            }),
            Transform::Static(Node::new("]".into())),
        ],
    );
}

fn transform_jsr_indirect_x(map: &mut TransformMap) {
    map.insert(
        JSR_INDIRECT_X.into(),
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

fn transform_long(map: &mut TransformMap) {
    map.insert(
        LONG.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" ".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U24,
                ..Default::default()
            }),
        ],
    );
}

fn transform_long_x(map: &mut TransformMap) {
    map.insert(
        LONG_X.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" ".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U24,
                ..Default::default()
            }),
            Transform::Static(Node::new(", x".into())),
        ],
    );
}

fn transform_indirect_y24(map: &mut TransformMap) {
    map.insert(
        INDIRECT_Y24.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" [".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
                ..Default::default()
            }),
            Transform::Static(Node::new("], y".into())),
        ],
    );
}

fn transform_relative16(map: &mut TransformMap) {
    map.insert(
        RELATIVE16.into(),
        vec![
            Transform::MatcherName,
            Transform::Static(Node::new(" ".into())),
            Transform::OffsetAddress(3),
            Transform::Val(ValOut {
                offset: 1,
                fmt: ValueTypeFmt::LowerHex(4),
                data_type: DataType::I16,
                rel: true,
                ..Default::default()
            }),
            Transform::OffsetAddress(-3),
            Transform::Consume(1),
        ],
    );
}

fn transform_move(map: &mut TransformMap) {
    map.insert(
        MOVE.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" #".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
                ..Default::default()
            }),
            Transform::Static(Node::new(", #".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
                ..Default::default()
            }),
        ],
    );
}

pub(super) fn transforms() -> TransformMap {
    let mut map = super::a65c02::transforms();
    transform_stack_s(&mut map);
    transform_direct24(&mut map);
    transform_long(&mut map);
    transform_stack_s_y(&mut map);
    transform_indirect_y24(&mut map);
    transform_long_x(&mut map);
    transform_relative16(&mut map);
    transform_move(&mut map);
    transform_jump_long_indirect(&mut map);
    transform_jsr_indirect_x(&mut map);
    map
}

fn matcher_stack_s(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, STACK_S);
}

fn matcher_direct24(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, DIRECT24);
}

fn matcher_long(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, LONG);
}

fn matcher_long_x(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, LONG_X);
}

fn matcher_stack_s_y(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, STACK_S_Y);
}

fn matcher_indirect_y24(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, INDIRECT_Y24);
}

fn matcher_relative16(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, RELATIVE16);
}

fn matcher_move(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, MOVE);
}

fn matcher_jump_long_indirect(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, JUMP_LONG_INDIRECT);
}

fn matcher_jsr_indirect_x(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, JSR_INDIRECT_X);
}

pub(super) fn matchers_from(matchers: &mut MatcherList, instrs: InstructionMap) {
    for (k, modes) in instrs.iter() {
        if let Some(op) = modes.get(STACK_S) {
            matcher_stack_s(matchers, *op, k);
        }
        if let Some(op) = modes.get(DIRECT24) {
            matcher_direct24(matchers, *op, k);
        }
        if let Some(op) = modes.get(LONG) {
            matcher_long(matchers, *op, k);
        }
        if let Some(op) = modes.get(LONG_X) {
            matcher_long_x(matchers, *op, k);
        }
        if let Some(op) = modes.get(STACK_S_Y) {
            matcher_stack_s_y(matchers, *op, k);
        }
        if let Some(op) = modes.get(INDIRECT_Y24) {
            matcher_indirect_y24(matchers, *op, k);
        }
        if let Some(op) = modes.get(RELATIVE16) {
            matcher_relative16(matchers, *op, k);
        }
        if let Some(op) = modes.get(MOVE) {
            matcher_move(matchers, *op, k);
        }
        if let Some(op) = modes.get(JUMP_LONG_INDIRECT) {
            matcher_jump_long_indirect(matchers, *op, k);
        }
        if let Some(op) = modes.get(JSR_INDIRECT_X) {
            matcher_jsr_indirect_x(matchers, *op, k);
        }
    }
    super::a65c02::matchers_from(matchers, instrs);
}

fn new_modes_instruction_map(
    name: &'static str,
    stack_s: u8,
    direct24: u8,
    long: u8,
    stack_s_y: u8,
    indirect_y24: u8,
    long_x: u8,
) -> (&'static str, ModeMap) {
    (
        name,
        ModeMap::from([
            (STACK_S, stack_s),
            (DIRECT24, direct24),
            (LONG, long),
            (STACK_S_Y, stack_s_y),
            (INDIRECT_Y24, indirect_y24),
            (LONG_X, long_x),
        ]),
    )
}

fn instruction_map() -> InstructionMap {
    InstructionMap::from([
        new_modes_instruction_map("ora", 0x03, 0x07, 0x0F, 0x13, 0x17, 0x1F),
        new_modes_instruction_map("and", 0x23, 0x27, 0x2F, 0x33, 0x37, 0x3F),
        new_modes_instruction_map("eor", 0x43, 0x47, 0x4F, 0x53, 0x57, 0x5F),
        new_modes_instruction_map("adc", 0x63, 0x67, 0x6F, 0x73, 0x77, 0x7F),
        new_modes_instruction_map("sta", 0x83, 0x87, 0x8F, 0x93, 0x97, 0x9F),
        new_modes_instruction_map("lda", 0xA3, 0xA7, 0xAF, 0xB3, 0xB7, 0xBF),
        new_modes_instruction_map("cmp", 0xC3, 0xC7, 0xCF, 0xD3, 0xD7, 0xDF),
        new_modes_instruction_map("sbc", 0xE3, 0xE7, 0xEF, 0xF3, 0xF7, 0xFF),
        implied_instruction_map("phd", 0x0B),
        implied_instruction_map("pld", 0x2B),
        implied_instruction_map("phk", 0x4B),
        implied_instruction_map("rtl", 0x6B),
        implied_instruction_map("phb", 0x8B),
        implied_instruction_map("plb", 0xAB),
        implied_instruction_map("wai", 0xCB),
        implied_instruction_map("xba", 0xEB),
        implied_instruction_map("tcs", 0x1B),
        implied_instruction_map("tsc", 0x3B),
        implied_instruction_map("tcd", 0x5B),
        implied_instruction_map("tdc", 0x7B),
        implied_instruction_map("txy", 0xBB),
        implied_instruction_map("tyx", 0xBB),
        implied_instruction_map("stp", 0xDB),
        implied_instruction_map("xce", 0xFB),
        ("cop", ModeMap::from([(IMMEDIATE, 0x02)])),
        ("jsl", ModeMap::from([(LONG, 0x22)])),
        implied_instruction_map("wdm", 0x42),
        ("per", ModeMap::from([(ABSOLUTE, 0x62)])),
        ("brl", ModeMap::from([(RELATIVE16, 0x82)])),
        ("rep", ModeMap::from([(IMMEDIATE, 0xC2)])),
        ("sep", ModeMap::from([(IMMEDIATE, 0xE2)])),
        ("mvn", ModeMap::from([(MOVE, 0x54)])),
        ("mvp", ModeMap::from([(MOVE, 0x44)])),
        ("pei", ModeMap::from([(IMMEDIATE, 0xD4)])),
        ("pea", ModeMap::from([(IMMEDIATE16, 0xF4)])),
        (
            "jmp",
            ModeMap::from([(LONG, 0x5C), (JUMP_LONG_INDIRECT, 0xDC)]),
        ),
        ("jsr", ModeMap::from([(JSR_INDIRECT_X, 0xFC)])),
    ])
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
            addr_type: DataType::U32,
            // we can unwrap this because the 6502 is guaranteed to have an empty
            // arch key!
            ..super::a6502::ARCH.archs.get("").unwrap().to_owned()
        },
    );
    map
}
