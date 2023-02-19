use std::collections::BTreeMap;

use crate::core::dasm::{arch::Archs, DataType, ValueTypeFmt};

use super::{
    AbsOut, Arch, Matcher, MatcherList, Node, NodeKind, Pattern, PatternAt, Transform,
    TransformList, TransformMap,
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
            Transform::Static(Node::new(" #$".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: short,
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
            Transform::Static(Node::new(" $".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
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
            Transform::Static(Node::new(" $".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
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
            Transform::Static(Node::new(" $".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U16,
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
            Transform::Static(Node::new(" $".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U16,
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
            Transform::Static(Node::new(" $".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U16,
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
            Transform::Static(Node::new(" ($".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
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
            Transform::Static(Node::new(" ($".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
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
}

fn transforms() -> TransformMap {
    let mut map = BTreeMap::default();

    map.insert(
        "define_byte".into(),
        vec![
            Transform::Static(Node::new(".db ".into())),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: crate::core::dasm::DataType::U8,
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

type MatcherFn = fn(&mut MatcherList, op: u8, name: &str, mode: &str);

/// creates matchers in the following order:
/// immediate, zp, zp_x, absolute, absolute_x, absolute_y, indirect_x, indirect_y
fn matcher_default_modes(
    matchers: &mut MatcherList,
    name: &str,
    ops: [u8; 8],
    immediate: MatcherFn,
) {
    immediate(matchers, ops[0], name, IMMEDIATE);
    matcher2(matchers, ops[1], name, ZP);
    matcher2(matchers, ops[2], name, ZP_X);
    matcher3(matchers, ops[3], name, ABSOLUTE);
    matcher3(matchers, ops[4], name, ABSOLUTE_X);
    matcher3(matchers, ops[5], name, ABSOLUTE_Y);
    matcher2(matchers, ops[6], name, INDIRECT_X);
    matcher2(matchers, ops[7], name, INDIRECT_Y);
}

fn matcher_logic(matchers: &mut MatcherList, name: &str, ops: [u8; 5]) {
    matcher1(matchers, ops[0], name, ACCUMULATOR);
    matcher2(matchers, ops[1], name, ZP);
    matcher2(matchers, ops[2], name, ZP_X);
    matcher3(matchers, ops[3], name, ABSOLUTE);
    matcher3(matchers, ops[4], name, ABSOLUTE_X);
}

fn patterns() -> MatcherList {
    let mut list = vec![];
    matcher_default_modes(
        &mut list,
        "adc",
        [0x69, 0x65, 0x75, 0x6D, 0x7D, 0x79, 0x61, 0x71],
        matcher2,
    );
    matcher_default_modes(
        &mut list,
        "and",
        [0x29, 0x25, 0x35, 0x2D, 0x3D, 0x39, 0x21, 0x31],
        matcher2,
    );
    matcher_logic(&mut list, "asl", [0x0A, 0x06, 0x16, 0x0E, 0x1E]);

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
            ..Arch::default()
        },
    );
    map
}
