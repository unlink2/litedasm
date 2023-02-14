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

fn transform_zp(map: &mut BTreeMap<String, TransformList>, name: &str) {
    map.insert(
        format!("{}_zp", name),
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

fn transform_zp_x(map: &mut BTreeMap<String, TransformList>, name: &str) {
    map.insert(
        format!("{}_zp_x", name),
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
    transform_zp(&mut map, "adc");
    transform_zp_x(&mut map, "adc");

    map
}

fn matcher2(op: u8, name: &str) -> Matcher {
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
        matcher2(0x69, "adc_immediate"),
        matcher2(0x65, "adc_zp"),
        matcher2(0x75, "adc_zp_x"),
        Matcher {
            patterns: vec![PatternAt::new(Pattern::Any, 0)],
            transforms: "define_byte".into(),
        },
    ]
}
