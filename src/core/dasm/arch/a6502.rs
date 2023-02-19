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
type Ops<'a> = (u8, &'a str);

fn add_matchers(matchers: &mut MatcherList, ops: &[Ops], mode: &str, f: MatcherFn) {
    ops.iter().for_each(|o| {
        f(matchers, o.0, o.1, mode);
    });
}

fn matcher_immediate(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, IMMEDIATE, matcher2);
}

fn matcher_zp(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, ZP, matcher2);
}

fn matcher_zp_x(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, ZP_X, matcher2);
}

fn matcher_absolute(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, ABSOLUTE, matcher3);
}

fn matcher_absolute_x(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, ABSOLUTE_X, matcher3);
}

fn matcher_absolute_y(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, ABSOLUTE_Y, matcher3);
}

fn matcher_indirect_x(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, INDIRECT_X, matcher2);
}

fn matcher_indirect_y(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, INDIRECT_Y, matcher2);
}

fn matcher_accumulator(matchers: &mut MatcherList, ops: &[Ops]) {
    add_matchers(matchers, ops, ACCUMULATOR, matcher1);
}

fn patterns() -> MatcherList {
    let mut list = vec![];

    matcher_accumulator(&mut list, &[(0x0A, "asl")]);
    matcher_immediate(&mut list, &[(0x69, "adc"), (0x29, "and")]);
    matcher_zp(
        &mut list,
        &[(0x65, "adc"), (0x25, "and"), (0x06, "asl"), (0x24, "bit")],
    );
    matcher_zp_x(&mut list, &[(0x75, "adc"), (0x35, "and"), (0x16, "asl")]);
    matcher_absolute(
        &mut list,
        &[(0x6D, "adc"), (0x2D, "and"), (0x0E, "asl"), (0x2C, "bit")],
    );
    matcher_absolute_x(&mut list, &[(0x7D, "adc"), (0x3D, "and"), (0x1E, "asl")]);
    matcher_absolute_y(&mut list, &[(0x79, "adc"), (0x39, "and")]);
    matcher_indirect_x(&mut list, &[(0x61, "adc"), (0x21, "and")]);
    matcher_indirect_y(&mut list, &[(0x71, "adc"), (0x32, "and")]);

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
