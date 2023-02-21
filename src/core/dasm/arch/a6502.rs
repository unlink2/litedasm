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
const INDIRECT_X: &str = "indirect_x";
const INDIRECT_Y: &str = "indirect_y";
const IMPLIED: &str = "implied";
const ACCUMULATOR: &str = "accumulator";
// addressing mode for branches
const RELATIVE: &str = "relative";

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

type ModeMap = BTreeMap<&'static str, u8>;
type InstructionMap = BTreeMap<&'static str, ModeMap>;

// creates a map of all isntructions and their respective
// modes
fn instruction_map() -> InstructionMap {
    InstructionMap::from([
        (
            "adc",
            ModeMap::from([
                (IMMEDIATE, 0x69),
                (ZP, 0x65),
                (ZP_X, 0x75),
                (ABSOLUTE, 0x6D),
                (ABSOLUTE_X, 0x7D),
                (ABSOLUTE_Y, 0x79),
                (INDIRECT_X, 0x61),
                (INDIRECT_Y, 0x71),
            ]),
        ),
        (
            "and",
            ModeMap::from([
                (IMMEDIATE, 0x29),
                (ZP, 0x25),
                (ZP_X, 0x35),
                (ABSOLUTE, 0x2D),
                (ABSOLUTE_X, 0x3D),
                (ABSOLUTE_Y, 0x39),
                (INDIRECT_X, 0x21),
                (INDIRECT_Y, 0x31),
            ]),
        ),
        (
            "asl",
            ModeMap::from([
                (ACCUMULATOR, 0x0A),
                (ZP, 0x06),
                (ZP_X, 0x16),
                (ABSOLUTE, 0x0E),
                (ABSOLUTE_X, 0x1E),
            ]),
        ),
        ("bit", ModeMap::from([(ZP, 0x24), (ZP_X, 0x2C)])),
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
            value_type_fmt_prefix: BTreeMap::from([
                (ValueTypeFmt::UpperHex(2), "$".into()),
                (ValueTypeFmt::LowerHex(2), "$".into()),
            ]),
            ..Arch::default()
        },
    );
    map
}
