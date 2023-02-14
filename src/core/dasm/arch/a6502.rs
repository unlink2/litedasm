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

fn transform_immediate(map: &mut BTreeMap<String, TransformList>, name: &str) {
    map.insert(
        format!("{}_immediate", name),
        vec![
            Transform::String("adc #$".into()),
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
    transform_immediate(&mut map, "adc");

    map
}

fn matcher1(op: u8, name: &str) -> Matcher {
    Matcher {
        patterns: vec![
            PatternAt::new(Pattern::Exact(op), 0),
            PatternAt::new(Pattern::Any, 1),
        ],
        transforms: name.into(),
    }
}

fn patterns() -> MatcherList {
    vec![
        matcher1(0x69, "adc_immediate"),
        Matcher {
            patterns: vec![PatternAt::new(Pattern::Any, 0)],
            transforms: "define_byte".into(),
        },
    ]
}
