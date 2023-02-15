use std::collections::BTreeMap;

use crate::core::dasm::{DataType, ValueTypeFmt};

use super::{AbsOut, Arch, Matcher, MatcherList, Pattern, PatternAt, Transform, TransformList};
use lazy_static::lazy_static;

lazy_static! {
    /// Built-in architecture for the 6502 family
    pub static ref ARCH: Arch = Arch {
        patterns: patterns(),
        transforms: transforms(),
        ..Arch::default()
    };
}

const IMMEDIATE: &str = "immediate";
const ZP: &str = "zp";
const ZP_X: &str = "zp_x";
const ABSOLUTE: &str = "absolute";
const ABSOLUTE_Y: &str = "absolute_y";
const ABSOLUTE_X: &str = "absolute_x";
const INDIRECT_X: &str = "indirect_x";
const INDIRECT_Y: &str = "indirect_y";

fn format_mode(name: &str, mode: &str) -> String {
    format!("{name}_{mode}")
}

fn transform_immediate(map: &mut BTreeMap<String, TransformList>, name: &str) {
    map.insert(
        format_mode(name, IMMEDIATE),
        vec![
            Transform::String(format!("{} #$", name)),
            Transform::Abs(AbsOut {
                offset: 1,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
            }),
            Transform::new_line(),
            Transform::Consume(1),
        ],
    );
}

fn transforms_immediate(map: &mut BTreeMap<String, TransformList>) {
    let names = ["adc"];

    names.iter().for_each(|n| transform_immediate(map, n));
}

fn transform_zp(map: &mut BTreeMap<String, TransformList>, name: &str) {
    map.insert(
        format_mode(name, ZP),
        vec![
            Transform::String(format!("{} $", name)),
            Transform::Abs(AbsOut {
                offset: 1,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
            }),
            Transform::new_line(),
            Transform::Consume(1),
        ],
    );
}

fn transforms_zp(map: &mut BTreeMap<String, TransformList>) {
    let names = ["adc"];
    names.iter().for_each(|n| transform_zp(map, n));
}

fn transform_zp_x(map: &mut BTreeMap<String, TransformList>, name: &str) {
    map.insert(
        format_mode(name, ZP_X),
        vec![
            Transform::String(format!("{} $", name)),
            Transform::Abs(AbsOut {
                offset: 1,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: DataType::U8,
            }),
            Transform::String(", x".into()),
            Transform::new_line(),
            Transform::Consume(1),
        ],
    );
}

fn transforms_zp_x(map: &mut BTreeMap<String, TransformList>) {
    let names = ["adc"];
    names.iter().for_each(|n| transform_zp_x(map, n));
}

fn transforms() -> BTreeMap<String, TransformList> {
    let mut map = BTreeMap::default();

    map.insert(
        "define_byte".into(),
        vec![
            Transform::String(".db ".into()),
            Transform::Abs(AbsOut {
                offset: 0,
                fmt: ValueTypeFmt::LowerHex(2),
                data_type: crate::core::dasm::DataType::U8,
            }),
            Transform::new_line(),
        ],
    );
    transforms_immediate(&mut map);
    transforms_zp(&mut map);
    transforms_zp_x(&mut map);

    map
}

fn matcher2(matchers: &mut MatcherList, op: u8, name: &str, mode: &str) {
    matchers.push(Matcher {
        patterns: vec![
            PatternAt::new(Pattern::Exact(op), 0),
            PatternAt::new(Pattern::Any, 1),
        ],
        transforms: format!("{name}_{mode}"),
    })
}

fn matcher3(matchers: &mut MatcherList, op: u8, name: &str, mode: &str) {
    matchers.push(Matcher {
        patterns: vec![
            PatternAt::new(Pattern::Exact(op), 0),
            PatternAt::new(Pattern::Any, 2),
        ],
        transforms: name.into(),
    })
}

fn matcher_default_modes(matchers: &mut MatcherList, name: &str, ops: [u8; 8]) {
    matcher2(matchers, ops[0], name, IMMEDIATE);
    matcher2(matchers, ops[1], name, ZP);
    matcher2(matchers, ops[2], name, ZP_X);
    matcher3(matchers, ops[3], name, ABSOLUTE);
    matcher3(matchers, ops[4], name, ABSOLUTE_X);
    matcher3(matchers, ops[5], name, ABSOLUTE_Y);
    matcher2(matchers, ops[6], name, INDIRECT_X);
    matcher2(matchers, ops[7], name, INDIRECT_Y);
}

fn patterns() -> MatcherList {
    let mut list = vec![];
    matcher_default_modes(
        &mut list,
        "adc",
        [0x69, 0x65, 0x75, 0x6D, 0x7D, 0x79, 0x61, 0x71],
    );
    list.push(Matcher {
        patterns: vec![PatternAt::new(Pattern::Any, 0)],
        transforms: "define_byte".into(),
    });
    list
}
