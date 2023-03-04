use std::fmt::Display;

use crate::prelude::FdResult;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use self::arch::{Arch, Node, NodeKind};
use lazy_static::lazy_static;

pub mod arch;
pub mod patch;
pub mod symbols;

pub type Address = u64;

/// All possible data types a transform can tranform into
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, PartialOrd, PartialEq, Ord, Eq, Copy, Clone, Debug)]
pub enum DataType {
    U8,
    U16,
    U24,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    #[default]
    None,
}

impl DataType {
    pub fn data_len(&self) -> usize {
        match self {
            DataType::U8 => 1,
            DataType::U16 => 2,
            DataType::U24 => 3,
            DataType::U32 => 4,
            DataType::U64 => 8,
            DataType::I8 => 1,
            DataType::I16 => 2,
            DataType::I32 => 4,
            DataType::I64 => 8,
            DataType::None => 0,
        }
    }

    pub fn mask(&self) -> ValueType {
        match self {
            DataType::U8 | DataType::I8 => 0xFF as ValueType,
            DataType::U16 | DataType::I16 => 0xFFFF as ValueType,
            DataType::U32 | DataType::I32 => 0xFFFFFFFF as ValueType,
            DataType::U64 | DataType::I64 => -1 as ValueType,
            DataType::U24 => 0xFFF,
            DataType::None => 0,
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValueTypeFmt {
    Binary(usize),
    Decimal(usize),
    LowerHex(usize),
    Octal(usize),
    UpperHex(usize),
}

impl Default for ValueTypeFmt {
    fn default() -> Self {
        Self::LowerHex(0)
    }
}

impl Display for ValueTypeFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueTypeFmt::Binary(_) => write!(f, "bin"),
            ValueTypeFmt::Decimal(_) => write!(f, "dec"),
            ValueTypeFmt::LowerHex(_) => write!(f, "hex"),
            ValueTypeFmt::Octal(_) => write!(f, "oct"),
            ValueTypeFmt::UpperHex(_) => write!(f, "HEX"),
        }
    }
}

impl ValueTypeFmt {
    pub fn post(&self) -> &str {
        match self {
            ValueTypeFmt::Binary(_) => "fmt_bin_post",
            ValueTypeFmt::Decimal(_) => "fmt_dec_post",
            ValueTypeFmt::LowerHex(_) => "fmt_hex_post",
            ValueTypeFmt::Octal(_) => "fmt_oct_post",
            ValueTypeFmt::UpperHex(_) => "fmt_HEX_post",
        }
    }

    pub fn pre(&self) -> &str {
        match self {
            ValueTypeFmt::Binary(_) => "fmt_bin_pre",
            ValueTypeFmt::Decimal(_) => "fmt_dec_pre",
            ValueTypeFmt::LowerHex(_) => "fmt_hex_pre",
            ValueTypeFmt::Octal(_) => "fmt_oct_pre",
            ValueTypeFmt::UpperHex(_) => "fmt_HEX_pre",
        }
    }
}

// The corresponding data type holding a value
pub type ValueType = i64;

macro_rules! format_value_type {
    ($val: expr, $fmt:expr, $pre:expr, $post:expr) => {
        match $fmt {
            ValueTypeFmt::Binary(width) => {
                Ok(Node::new(format!("{}{:0width$b}{}", $pre, $val, $post)))
            }
            ValueTypeFmt::Decimal(width) => {
                Ok(Node::new(format!("{}{:0width$}{}", $pre, $val, $post)))
            }
            ValueTypeFmt::LowerHex(width) => {
                Ok(Node::new(format!("{}{:0width$x}{}", $pre, $val, $post)))
            }
            ValueTypeFmt::Octal(width) => {
                Ok(Node::new(format!("{}{:0width$o}{}", $pre, $val, $post)))
            }
            ValueTypeFmt::UpperHex(width) => {
                Ok(Node::new(format!("{}{:0width$X}{}", $pre, $val, $post)))
            } // _ => Err(Error::UnsupportedFormat($fmt)),
        }
    };
}

lazy_static! {
    static ref EMPTY_NODE: Node = Node::new(String::new());
}

pub fn try_to_node(v: ValueType, fmt: ValueTypeFmt, arch: &Arch) -> FdResult<Node> {
    let pre = arch.node_map.get(fmt.pre()).unwrap_or(&EMPTY_NODE);
    let post = arch.node_map.get(fmt.post()).unwrap_or(&EMPTY_NODE);
    let node: FdResult<Node> = format_value_type!(v, fmt, pre, post);
    let mut node = node?;
    node.kind = NodeKind::Value(v);
    Ok(node)
}

#[cfg(test)]
mod test {
    use super::{
        arch::{a6502, a65c02, a65c816, Context},
        symbols::{Symbol, SymbolKind},
        Address,
    };
    use crate::core::dasm::arch::Archs;

    fn test_arch_result(arch: &Archs, data: &[u8], expected: &str, end_addr: Address) {
        let mut result = "".to_string();
        let ctx = arch
            .disas(
                |n, _raw, _arch, _ctx| {
                    result.push_str(&n.string);
                    Ok(())
                },
                data,
            )
            .unwrap();

        assert_eq!(expected, result);
        assert_eq!(end_addr, ctx.address())
    }

    fn test_arch_result_ctx(
        arch: &Archs,
        ctx: &mut Context,
        data: &[u8],
        expected: &str,
        end_addr: Address,
    ) {
        let mut result = "".to_string();
        arch.disas_ctx(
            |n, _raw, _arch, _ctx| {
                result.push_str(&n.string);
                Ok(())
            },
            data,
            ctx,
        )
        .unwrap();

        assert_eq!(expected, result);
        assert_eq!(end_addr, ctx.address())
    }

    #[test]
    fn a6502() {
        // byte and immediate
        test_arch_result(
            &a6502::ARCH,
            &[0xFF, 0xab, 0x69, 0x02, 0x01],
            "00000000 .db $ff\n00000001 .db $ab\n00000002 adc #$02\n00000004 .db $01\n",
            0x5,
        );

        // immediate_x flag
        test_arch_result(&a6502::ARCH, &[0xA2, 0x12], "00000000 ldx #$12\n", 2);

        // zero page, x
        test_arch_result(&a6502::ARCH, &[0x75, 0x12], "00000000 adc $12, x\n", 2);

        // zero page
        test_arch_result(&a6502::ARCH, &[0x65, 0x12], "00000000 adc $12\n", 2);

        // absolute
        test_arch_result(&a6502::ARCH, &[0x6D, 0x34, 0x12], "00000000 adc $1234\n", 3);

        // absolute, x
        test_arch_result(
            &a6502::ARCH,
            &[0x7D, 0x34, 0x12],
            "00000000 adc $1234, x\n",
            3,
        );

        // absolute, y
        test_arch_result(
            &a6502::ARCH,
            &[0x79, 0x34, 0x12],
            "00000000 adc $1234, y\n",
            3,
        );

        // indirect, x
        test_arch_result(&a6502::ARCH, &[0x61, 0x12], "00000000 adc ($12, x)\n", 2);

        // indirect, y
        test_arch_result(&a6502::ARCH, &[0x71, 0x12], "00000000 adc ($12), y\n", 2);

        // accumulator
        test_arch_result(&a6502::ARCH, &[0x0A], "00000000 asl a\n", 1);

        // relative
        test_arch_result(&a6502::ARCH, &[0x10, 0x11], "00000000 bpl $11\n", 2);

        // implied
        test_arch_result(&a6502::ARCH, &[0x00], "00000000 brk\n", 1);

        // indirect
        test_arch_result(
            &a6502::ARCH,
            &[0x6C, 0x34, 0x12],
            "00000000 jmp ($1234)\n",
            3,
        );

        // zp, y
        test_arch_result(&a6502::ARCH, &[0xB6, 0x12], "00000000 ldx $12, y\n", 2);
    }

    #[test]
    fn a65c02() {
        // ora immediate
        test_arch_result(&a65c02::ARCH, &[0x09, 0x12], "00000000 ora #$12\n", 2);

        // ora indirect
        test_arch_result(&a65c02::ARCH, &[0x12, 0x12], "00000000 ora ($12)\n", 2);

        // jmp (abs, x)
        test_arch_result(
            &a65c02::ARCH,
            &[0x7C, 0x34, 0x12],
            "00000000 jmp ($1234, x)\n",
            3,
        );
    }

    #[test]
    fn a65c816() {
        // immediate m-flag set
        {
            let mut ctx = Context::default();
            ctx.def_flag("m", "");

            test_arch_result_ctx(
                &a65c816::ARCH,
                &mut ctx,
                &[0xA2, 0x12, 0xA9, 0x34, 0x12],
                "00000000 ldx #$12\n00000002 lda #$1234\n",
                5,
            );
        }
        // immediate x-flag set

        {
            let mut ctx = Context::default();
            ctx.def_flag("x", "");

            test_arch_result_ctx(
                &a65c816::ARCH,
                &mut ctx,
                &[0xA2, 0x34, 0x12, 0xA9, 0x34],
                "00000000 ldx #$1234\n00000003 lda #$34\n",
                5,
            );
        }

        // stack, S
        test_arch_result(&a65c816::ARCH, &[0x03, 0x12], "00000000 ora $12, s\n", 2);

        // [dp]
        test_arch_result(&a65c816::ARCH, &[0x07, 0x12], "00000000 ora [$12]\n", 2);

        // long + test data read size override
        test_arch_result(
            &a65c816::ARCH,
            &[0x0F, 0x56, 0x34, 0x12, 0xEA],
            "00000000 ora $123456\n00000004 nop\n",
            5,
        );

        // (stack, S), Y
        test_arch_result(
            &a65c816::ARCH,
            &[0x13, 0x12],
            "00000000 ora ($12, s), y\n",
            2,
        );

        // [dp], y
        test_arch_result(&a65c816::ARCH, &[0x17, 0x12], "00000000 ora [$12], y\n", 2);

        // long, x
        test_arch_result(
            &a65c816::ARCH,
            &[0x1F, 0x56, 0x34, 0x12],
            "00000000 ora $123456, x\n",
            4,
        );

        // cop
        test_arch_result(&a65c816::ARCH, &[0x02, 0x12], "00000000 cop #$12\n", 2);

        // jsl with label
        {
            let mut ctx = Context::default();

            ctx.def_symbol(Symbol::new(
                "test".into(),
                SymbolKind::Label,
                super::symbols::Scope::Global,
                0x123456,
                1,
            ));

            test_arch_result_ctx(
                &a65c816::ARCH,
                &mut ctx,
                &[0x22, 0x56, 0x34, 0x12, 0x22, 0x12, 0x34, 0x56],
                "00000000 jsl test\n00000004 jsl $563412\n",
                8,
            );
        }

        // brl
        test_arch_result(
            &a65c816::ARCH,
            &[0x82, 0x34, 0x12],
            "00000000 brl $1234\n",
            3,
        );

        // brl with label
        {
            let mut ctx = Context::default();

            ctx.def_symbol(Symbol::new(
                "test".into(),
                SymbolKind::Label,
                super::symbols::Scope::Global,
                0x05,
                1,
            ));

            test_arch_result_ctx(
                &a65c816::ARCH,
                &mut ctx,
                &[0x82, 0x02, 0x00],
                "00000000 brl test\n",
                3,
            );
        }

        // jmp with label
        {
            let mut ctx = Context::default();

            ctx.def_symbol(Symbol::new(
                "test".into(),
                SymbolKind::Label,
                super::symbols::Scope::Global,
                0x1234,
                1,
            ));

            test_arch_result_ctx(
                &a65c816::ARCH,
                &mut ctx,
                &[0x4C, 0x34, 0x12],
                "00000000 jmp test\n",
                3,
            );
        }

        // move
        test_arch_result(
            &a65c816::ARCH,
            &[0x54, 0x12, 0x34],
            "00000000 mvn #$12, #$34\n",
            3,
        );

        // pea
        test_arch_result(
            &a65c816::ARCH,
            &[0xF4, 0x34, 0x12],
            "00000000 pea #$1234\n",
            3,
        );

        // jmp [long]
        test_arch_result(
            &a65c816::ARCH,
            &[0xDC, 0x34, 0x12],
            "00000000 jmp [$1234]\n",
            3,
        );

        // jsr (abs, x)
        test_arch_result(
            &a65c816::ARCH,
            &[0xFC, 0x34, 0x12],
            "00000000 jsr ($1234, x)\n",
            3,
        );
    }

    #[test]
    fn labels() {
        let mut ctx = Context::default();
        ctx.def_symbol(Symbol::new(
            "test".into(),
            SymbolKind::Label,
            super::symbols::Scope::Global,
            0x2,
            1,
        ));
        ctx.def_symbol(Symbol::new(
            "test2".into(),
            SymbolKind::Label,
            super::symbols::Scope::Global,
            0xE,
            1,
        ));
        ctx.def_symbol(Symbol::new(
            "scoped_test_out".into(),
            SymbolKind::Label,
            super::symbols::Scope::Range(0x00, 0x01),
            0x2,
            1,
        ));
        ctx.def_symbol(Symbol::new(
            "scoped_test_in".into(),
            SymbolKind::Label,
            super::symbols::Scope::Range(0x01, 0x03),
            0x2,
            1,
        ));
        ctx.def_symbol(Symbol::new(
            "const_test".into(),
            SymbolKind::Const,
            super::symbols::Scope::Global,
            0x2,
            1,
        ));
        ctx.def_symbol(Symbol::new(
            "len_test".into(),
            SymbolKind::Const,
            super::symbols::Scope::Global,
            0x10,
            3,
        ));

        ctx.def_symbol(Symbol::new(
            "type_test".into(),
            SymbolKind::Label,
            super::symbols::Scope::Range(0x00, 0x01),
            0x2,
            1,
        ));
        test_arch_result_ctx(
            &a6502::ARCH,
            &mut ctx,
            &[0x00, 0x00, 0x00, 0x00, 0x10, (-4_i8) as u8],
            "00000000 brk\n00000001 brk\ntest:\nscoped_test_in:\n00000002 brk\n00000003 brk\n00000004 bpl test\n",
            6,
        );

        test_arch_result_ctx(
            &a6502::ARCH,
            &mut ctx,
            &[0x4C, 0x0E, 0x00, 0xEA, 0x10, (2_i8) as u8, 0xA9, 0x2],
            "00000006 jmp test2\n00000009 nop\n0000000a bpl test2\n0000000c lda #test\n",
            14,
        );

        test_arch_result_ctx(
            &a6502::ARCH,
            &mut ctx,
            &[0x69, 0x11, 0x69, 0x12, 0x69, 0x13],
            "test2:\n0000000e adc #len_test+1\n00000010 adc #len_test+2\n00000012 adc #$13\n",
            20,
        );
    }
}
