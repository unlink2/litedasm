use std::collections::BTreeMap;

use crate::core::dasm::{arch::Archs, DataType, ValueTypeFmt};

use super::{
    Arch, Matcher, MatcherList, Node, NodeKind, Pattern, PatternAt, Transform, TransformList,
    TransformMap, ValOut,
};
use lazy_static::lazy_static;

lazy_static! {
    /// Built-in architecture for the 6502 family
    pub static ref ARCH: Archs = Archs {archs: archs(), ..Default::default()};
}

const IMMEDIATE: &str = "immediate";
const ZP: &str = "zp";
const ZP_X: &str = "zp_x";
const ABSOLUTE: &str = "absolute";
const ABSOLUTE_Y: &str = "absolute_y";
const ABSOLUTE_X: &str = "absolute_x";
const INDIRECT: &str = "indirect";
const INDIRECT_X: &str = "indirect_x";
const INDIRECT_Y: &str = "indirect_y";
const IMPLIED: &str = "implied";
const ACCUMULATOR: &str = "accumulator";
// addressing mode for branches
const RELATIVE: &str = "relative";

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
                data_type: DataType::U16,
                ..Default::default()
            }),
            Transform::Static(Node::new(")".into())),
        ],
    );
}

fn transform_accumulator(map: &mut TransformMap) {
    map.insert(
        ACCUMULATOR.to_owned(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" a".into())),
        ],
    );
}

fn transform_immediate(map: &mut TransformMap, short: DataType) {
    map.insert(
        IMMEDIATE.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" #".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: short,
                ..Default::default()
            }),
        ],
    );
}

fn transform_relative(map: &mut TransformMap) {
    map.insert(
        RELATIVE.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" ".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
                rel: true,
                ..Default::default()
            }),
        ],
    );
}

fn transform_zp(map: &mut TransformMap) {
    map.insert(
        ZP.into(),
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
        ],
    );
}

fn transform_zp_x(map: &mut TransformMap) {
    map.insert(
        ZP_X.into(),
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
            Transform::Static(Node::new(", x".into())),
        ],
    );
}

fn transform_absolute(map: &mut TransformMap) {
    map.insert(
        ABSOLUTE.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" ".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U16,
                ..Default::default()
            }),
        ],
    );
}

fn transform_absolute_x(map: &mut TransformMap) {
    map.insert(
        ABSOLUTE_X.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" ".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U16,
                ..Default::default()
            }),
            Transform::Static(Node::new(", x".into())),
        ],
    );
}

fn transform_absolute_y(map: &mut TransformMap) {
    map.insert(
        ABSOLUTE_Y.into(),
        vec![
            Transform::MatcherName,
            Transform::Consume(1),
            Transform::Static(Node::new(" ".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U16,
                ..Default::default()
            }),
            Transform::Static(Node::new(", y".into())),
        ],
    );
}

fn transform_indirect_x(map: &mut TransformMap) {
    map.insert(
        INDIRECT_X.into(),
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
            Transform::Static(Node::new(", x)".into())),
        ],
    );
}

fn transform_indirect_y(map: &mut TransformMap) {
    map.insert(
        INDIRECT_Y.into(),
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
            Transform::Static(Node::new("), y".into())),
        ],
    );
}

fn transform_implied(map: &mut TransformMap) {
    map.insert(
        IMPLIED.into(),
        vec![Transform::MatcherName, Transform::Consume(1)],
    );
}

fn transforms_default_modes(map: &mut TransformMap) {
    transform_immediate(map, DataType::U8);
    transform_zp(map);
    transform_zp_x(map);
    transform_absolute(map);
    transform_absolute_x(map);
    transform_absolute_y(map);
    transform_indirect_x(map);
    transform_indirect_y(map);
    transform_implied(map);
    transform_accumulator(map);
    transform_relative(map);
    transform_indirect(map);
}

fn transforms() -> TransformMap {
    let mut map = BTreeMap::default();

    map.insert(
        "define_byte".into(),
        vec![
            Transform::Static(Node::new(".db ".into())),
            Transform::Val(ValOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: crate::core::dasm::DataType::U8,
                ..Default::default()
            }),
        ],
    );
    transforms_default_modes(&mut map);

    map
}

fn matcher1(matchers: &mut MatcherList, op: u8, name: &str, mode: &str) {
    matchers.push(Matcher {
        patterns: vec![PatternAt::new(Pattern::Exact(op), 0)],
        transforms: mode.into(),
        name: Node::new(name.into()),
    })
}

fn matcher2(matchers: &mut MatcherList, op: u8, name: &str, mode: &str) {
    matchers.push(Matcher {
        patterns: vec![
            PatternAt::new(Pattern::Exact(op), 0),
            PatternAt::new(Pattern::Any, 1),
        ],
        transforms: mode.into(),
        name: Node::new(name.into()),
    })
}

fn matcher3(matchers: &mut MatcherList, op: u8, name: &str, mode: &str) {
    matchers.push(Matcher {
        patterns: vec![
            PatternAt::new(Pattern::Exact(op), 0),
            PatternAt::new(Pattern::Any, 2),
        ],
        transforms: mode.into(),
        name: Node::new(name.into()),
    })
}

fn matcher_immediate(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, IMMEDIATE);
}

fn matcher_zp(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, ZP);
}

fn matcher_zp_x(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, ZP_X);
}

fn matcher_absolute(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, ABSOLUTE);
}

fn matcher_absolute_x(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, ABSOLUTE_X);
}

fn matcher_absolute_y(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, ABSOLUTE_Y)
}

fn matcher_indirect_x(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, INDIRECT_X);
}

fn matcher_indirect_y(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, INDIRECT_Y);
}

fn matcher_accumulator(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher1(matchers, op, name, ACCUMULATOR);
}

fn matcher_relative(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, RELATIVE);
}

fn matcher_implied(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher1(matchers, op, name, IMPLIED);
}

fn matcher_indirect(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, INDIRECT);
}

type ModeMap = BTreeMap<&'static str, u8>;
type InstructionMap = BTreeMap<&'static str, ModeMap>;

fn relative_instruction_map(name: &'static str, opcode: u8) -> (&'static str, ModeMap) {
    (name, ModeMap::from([(RELATIVE, opcode)]))
}

fn default_instruction_map(
    name: &'static str,
    immediate: u8,
    zp: u8,
    zp_x: u8,
    abs: u8,
    abs_x: u8,
    abs_y: u8,
    ind_x: u8,
    ind_y: u8,
) -> (&'static str, ModeMap) {
    (
        name,
        ModeMap::from([
            (IMMEDIATE, immediate),
            (ZP, zp),
            (ZP_X, zp_x),
            (ABSOLUTE, abs),
            (ABSOLUTE_X, abs_x),
            (ABSOLUTE_Y, abs_y),
            (INDIRECT_X, ind_x),
            (INDIRECT_Y, ind_y),
        ]),
    )
}

fn compare_index_instruction_map(
    name: &'static str,
    immediate: u8,
    zp: u8,
    abs: u8,
) -> (&'static str, ModeMap) {
    (
        name,
        ModeMap::from([(IMMEDIATE, immediate), (ZP, zp), (ABSOLUTE, abs)]),
    )
}

fn accumulator_instruction_map(
    name: &'static str,
    accumulator: u8,
    zp: u8,
    zp_x: u8,
    abs: u8,
    abs_x: u8,
) -> (&'static str, ModeMap) {
    (
        name,
        ModeMap::from([
            (ACCUMULATOR, accumulator),
            (ZP, zp),
            (ZP_X, zp_x),
            (ABSOLUTE, abs),
            (ABSOLUTE_X, abs_x),
        ]),
    )
}

fn inc_dec_instruction_map(
    name: &'static str,
    zp: u8,
    zp_x: u8,
    abs: u8,
    abs_x: u8,
) -> (&'static str, ModeMap) {
    (
        name,
        ModeMap::from([(ZP, zp), (ZP_X, zp_x), (ABSOLUTE, abs), (ABSOLUTE_X, abs_x)]),
    )
}

fn implied_instruction_map(name: &'static str, op: u8) -> (&'static str, ModeMap) {
    (name, ModeMap::from([(IMPLIED, op)]))
}

// creates a map of all instructions and their respective modes
fn instruction_map() -> InstructionMap {
    InstructionMap::from([
        default_instruction_map("adc", 0x69, 0x65, 0x75, 0x6D, 0x7D, 0x79, 0x61, 0x71),
        default_instruction_map("and", 0x29, 0x25, 0x35, 0x2D, 0x3D, 0x39, 0x21, 0x31),
        accumulator_instruction_map("asl", 0x0A, 0x06, 0x16, 0x0E, 0x1E),
        ("bit", ModeMap::from([(ZP, 0x24), (ZP_X, 0x2C)])),
        relative_instruction_map("bpl", 0x10),
        relative_instruction_map("bmi", 0x30),
        relative_instruction_map("bvc", 0x50),
        relative_instruction_map("bvs", 0x70),
        relative_instruction_map("bcc", 0x90),
        relative_instruction_map("bcs", 0xB0),
        relative_instruction_map("bne", 0xD0),
        relative_instruction_map("beq", 0xF0),
        implied_instruction_map("brk", 0x00),
        default_instruction_map("cmd", 0xC9, 0xC5, 0xD5, 0xCD, 0xDD, 0xD9, 0xC1, 0xD1),
        compare_index_instruction_map("cpx", 0xE0, 0xE4, 0xEC),
        compare_index_instruction_map("cpy", 0xC0, 0xC4, 0xCC),
        inc_dec_instruction_map("dec", 0xC6, 0xD6, 0xCE, 0xDE),
        default_instruction_map("eor", 0x49, 0x45, 0x55, 0x4D, 0x5D, 0x59, 0x41, 0x51),
        implied_instruction_map("clc", 0x18),
        implied_instruction_map("sec", 0x38),
        implied_instruction_map("cli", 0x58),
        implied_instruction_map("sei", 0x78),
        implied_instruction_map("clv", 0xB8),
        implied_instruction_map("cld", 0xD8),
        implied_instruction_map("sed", 0xF8),
        inc_dec_instruction_map("inc", 0xE6, 0xF6, 0xEE, 0xFE),
        ("jmp", ModeMap::from([(ABSOLUTE, 0x4C), (INDIRECT, 0x6C)])),
        ("jsr", ModeMap::from([(ABSOLUTE, 0x4C)])),
    ])
}

// converts the instruction map to a list of matchers
fn matchers_from(matchers: &mut MatcherList, instrs: InstructionMap) {
    for (k, modes) in instrs.iter() {
        // map all keys to the respective calls
        if let Some(op) = modes.get(IMMEDIATE) {
            matcher_immediate(matchers, *op, k);
        }
        if let Some(op) = modes.get(ZP) {
            matcher_zp(matchers, *op, k);
        }
        if let Some(op) = modes.get(ZP_X) {
            matcher_zp_x(matchers, *op, k)
        }
        if let Some(op) = modes.get(ABSOLUTE) {
            matcher_absolute(matchers, *op, k);
        }
        if let Some(op) = modes.get(ABSOLUTE_X) {
            matcher_absolute_x(matchers, *op, k);
        }
        if let Some(op) = modes.get(ABSOLUTE_Y) {
            matcher_absolute_y(matchers, *op, k);
        }
        if let Some(op) = modes.get(INDIRECT_X) {
            matcher_indirect_x(matchers, *op, k);
        }
        if let Some(op) = modes.get(INDIRECT_Y) {
            matcher_indirect_y(matchers, *op, k);
        }
        if let Some(op) = modes.get(ACCUMULATOR) {
            matcher_accumulator(matchers, *op, k);
        }
        if let Some(op) = modes.get(RELATIVE) {
            matcher_relative(matchers, *op, k);
        }
        if let Some(op) = modes.get(IMPLIED) {
            matcher_implied(matchers, *op, k);
        }
        if let Some(op) = modes.get(INDIRECT) {
            matcher_indirect(matchers, *op, k);
        }
    }
}

fn patterns() -> MatcherList {
    let mut list = vec![];

    matchers_from(&mut list, instruction_map());

    list.push(Matcher {
        patterns: vec![PatternAt::new(Pattern::Any, 0)],
        transforms: "define_byte".into(),
        name: Node::new(".db".into()),
    });
    list
}

fn archs() -> BTreeMap<String, Arch> {
    let mut map = BTreeMap::default();
    map.insert(
        "".into(),
        Arch {
            patterns: patterns(),
            transforms: transforms(),
            pre_transforms: vec![Transform::Address(8), Transform::space(1)],
            post_transforms: vec![Transform::new_line()],
            node_map: BTreeMap::from([
                (
                    ValueTypeFmt::LowerHex(0).pre().into(),
                    Node::new("$".into()),
                ),
                (
                    ValueTypeFmt::UpperHex(0).pre().into(),
                    Node::new("$".into()),
                ),
            ]),

            ..Arch::default()
        },
    );
    map
}
