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

pub(super) const IMMEDIATE: &str = "immediate";
pub(super) const ZP: &str = "zp";
pub(super) const ZP_X: &str = "zp_x";
pub(super) const ZP_Y: &str = "zp_y";
pub(super) const ABSOLUTE: &str = "absolute";
pub(super) const ABSOLUTE_Y: &str = "absolute_y";
pub(super) const ABSOLUTE_X: &str = "absolute_x";
pub(super) const INDIRECT_JMP: &str = "indirect_jmp";
pub(super) const INDIRECT_X: &str = "indirect_x";
pub(super) const INDIRECT_Y: &str = "indirect_y";
pub(super) const IMPLIED: &str = "implied";
pub(super) const ACCUMULATOR: &str = "accumulator";
// addressing mode for brapubes
pub(super) const RELATIVE: &str = "relative";

fn transform_indirect_jmp(map: &mut TransformMap) {
    map.insert(
        INDIRECT_JMP.into(),
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
            Transform::Static(Node::new(" ".into())),
            Transform::OffsetAddress(2),
            Transform::Val(ValOut {
                offset: 1,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::I8,
                rel: true,
                ..Default::default()
            }),
            Transform::OffsetAddress(-2),
            Transform::Consume(1),
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

fn transform_zp_y(map: &mut TransformMap) {
    map.insert(
        ZP_Y.into(),
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
            Transform::Static(Node::new(", y".into())),
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
    transform_zp_y(map);
    transform_absolute(map);
    transform_absolute_x(map);
    transform_absolute_y(map);
    transform_indirect_x(map);
    transform_indirect_y(map);
    transform_implied(map);
    transform_accumulator(map);
    transform_relative(map);
    transform_indirect_jmp(map);
}

pub(super) fn transforms() -> TransformMap {
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

pub(super) fn matcher1(matchers: &mut MatcherList, op: u8, name: &str, mode: &str) {
    matchers.push(Matcher {
        patterns: vec![PatternAt::new(Pattern::Exact(op), 0)],
        transforms: mode.into(),
        name: Node::new(name.into()),
    })
}

pub(super) fn matcher2(matchers: &mut MatcherList, op: u8, name: &str, mode: &str) {
    matchers.push(Matcher {
        patterns: vec![
            PatternAt::new(Pattern::Exact(op), 0),
            PatternAt::new(Pattern::Any, 1),
        ],
        transforms: mode.into(),
        name: Node::new(name.into()),
    })
}

pub(super) fn matcher3(matchers: &mut MatcherList, op: u8, name: &str, mode: &str) {
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

fn matcher_zp_y(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher2(matchers, op, name, ZP_Y);
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

fn matcher_indirect_jmp(matchers: &mut MatcherList, op: u8, name: &str) {
    matcher3(matchers, op, name, INDIRECT_JMP);
}

pub(super) type ModeMap = BTreeMap<&'static str, u8>;
pub(super) type InstructionMap = BTreeMap<&'static str, ModeMap>;

pub(super) fn relative_instruction_map(name: &'static str, opcode: u8) -> (&'static str, ModeMap) {
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

pub(super) fn implied_instruction_map(name: &'static str, op: u8) -> (&'static str, ModeMap) {
    (name, ModeMap::from([(IMPLIED, op)]))
}

// creates a map of all instructions and their respective modes
fn instruction_map() -> InstructionMap {
    InstructionMap::from([
        default_instruction_map("adc", 0x69, 0x65, 0x75, 0x6D, 0x7D, 0x79, 0x61, 0x71),
        default_instruction_map("and", 0x29, 0x25, 0x35, 0x2D, 0x3D, 0x39, 0x21, 0x31),
        accumulator_instruction_map("asl", 0x0A, 0x06, 0x16, 0x0E, 0x1E),
        ("bit", ModeMap::from([(ABSOLUTE, 0x24), (ZP, 0x2C)])),
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
        (
            "jmp",
            ModeMap::from([(ABSOLUTE, 0x4C), (INDIRECT_JMP, 0x6C)]),
        ),
        ("jsr", ModeMap::from([(ABSOLUTE, 0x4C)])),
        default_instruction_map("lda", 0xA9, 0xA5, 0xB5, 0xAD, 0xBD, 0xB9, 0xA1, 0xB1),
        (
            "ldx",
            ModeMap::from([
                (IMMEDIATE, 0xA2),
                (ZP, 0xA6),
                (ZP_Y, 0xB6),
                (ABSOLUTE, 0xAE),
                (ABSOLUTE_Y, 0xBE),
            ]),
        ),
        (
            "ldy",
            ModeMap::from([
                (IMMEDIATE, 0xA0),
                (ZP, 0xA4),
                (ZP_X, 0xB4),
                (ABSOLUTE, 0xAC),
                (ABSOLUTE_X, 0xBC),
            ]),
        ),
        accumulator_instruction_map("lsr", 0x4A, 0x46, 0x56, 0x4E, 0x5E),
        implied_instruction_map("nop", 0xEA),
        default_instruction_map("ora", 0x09, 0x05, 0x15, 0x0D, 0x1D, 0x19, 0x01, 0x11),
        implied_instruction_map("tax", 0xAA),
        implied_instruction_map("txa", 0xBA),
        implied_instruction_map("dex", 0xCA),
        implied_instruction_map("inx", 0xE8),
        implied_instruction_map("tay", 0xA8),
        implied_instruction_map("tya", 0x98),
        implied_instruction_map("dey", 0x88),
        implied_instruction_map("iny", 0xC8),
        accumulator_instruction_map("rol", 0x2A, 0x26, 0x36, 0x2E, 0x3E),
        accumulator_instruction_map("ror", 0x6A, 0x66, 0x76, 0x6E, 0x7E),
        implied_instruction_map("rti", 0x40),
        implied_instruction_map("rts", 0x60),
        default_instruction_map("sbc", 0xE9, 0xE5, 0xF5, 0xED, 0xFD, 0xF9, 0xE1, 0xF1),
        (
            "sta",
            ModeMap::from([
                (ZP, 0x85),
                (ZP_X, 0x95),
                (ABSOLUTE, 0x8D),
                (ABSOLUTE_X, 0x9D),
                (ABSOLUTE_Y, 0x99),
                (INDIRECT_X, 0x81),
                (INDIRECT_Y, 0x91),
            ]),
        ),
        implied_instruction_map("txs", 0x9A),
        implied_instruction_map("tsx", 0xBA),
        implied_instruction_map("pha", 0x48),
        implied_instruction_map("pla", 0x68),
        implied_instruction_map("php", 0x08),
        implied_instruction_map("plp", 0x28),
        (
            "stx",
            ModeMap::from([(ZP, 0x86), (ZP_Y, 0x96), (ABSOLUTE, 0x8E)]),
        ),
        (
            "sty",
            ModeMap::from([(ZP, 0x84), (ZP_X, 0x94), (ABSOLUTE, 0x8C)]),
        ),
    ])
}

// converts the instruction map to a list of matchers
pub(super) fn matchers_from(matchers: &mut MatcherList, instrs: InstructionMap) {
    for (k, modes) in instrs.iter() {
        // FIXME this is awful to read
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

        if let Some(op) = modes.get(ZP_Y) {
            matcher_zp_y(matchers, *op, k)
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

        if let Some(op) = modes.get(INDIRECT_JMP) {
            matcher_indirect_jmp(matchers, *op, k);
        }
    }
}

pub(super) fn patterns() -> MatcherList {
    let mut list = vec![];

    matchers_from(&mut list, instruction_map());

    list
}

pub(super) fn add_patterns_default(mut list: MatcherList) -> MatcherList {
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
            patterns: add_patterns_default(patterns()),
            transforms: transforms(),
            pre_transforms: vec![Transform::Label, Transform::Address(8), Transform::space(1)],
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
            addr_type: DataType::U16,
            ..Arch::default()
        },
    );
    map
}
