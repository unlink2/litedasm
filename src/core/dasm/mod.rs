use std::fmt::Display;

use crate::prelude::{Error, FdResult};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use self::arch::{Arch, Node, NodeKind};
use lazy_static::lazy_static;

pub mod arch;
pub mod symbols;

pub type Address = u64;

/// All possible data types a transform can tranform into
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, PartialOrd, PartialEq, Ord, Eq, Copy, Clone)]
pub enum DataType {
    U8,
    U16,
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
    pub fn is_in_address_space(&self, addr: Address) {
        todo!()
    }

    pub fn data_len(&self) -> usize {
        match self {
            DataType::U8 => 1,
            DataType::U16 => 2,
            DataType::U32 => 4,
            DataType::U64 => 8,
            DataType::I8 => 1,
            DataType::I16 => 2,
            DataType::I32 => 4,
            DataType::I64 => 8,
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
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Default, PartialOrd, PartialEq, Ord, Eq, Copy, Clone)]
pub enum ValueType {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    #[default]
    None,
}

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

impl ValueType {
    pub fn try_to_node(&self, fmt: ValueTypeFmt, arch: &Arch) -> FdResult<Node> {
        let pre = arch.node_map.get(fmt.pre()).unwrap_or(&EMPTY_NODE);
        let post = arch.node_map.get(fmt.post()).unwrap_or(&EMPTY_NODE);
        let node: FdResult<Node> = match self {
            ValueType::U8(v) => format_value_type!(v, fmt, pre, post),
            ValueType::U16(v) => format_value_type!(v, fmt, pre, post),
            ValueType::U32(v) => format_value_type!(v, fmt, pre, post),
            ValueType::U64(v) => format_value_type!(v, fmt, pre, post),
            ValueType::I8(v) => format_value_type!(v, fmt, pre, post),
            ValueType::I16(v) => format_value_type!(v, fmt, pre, post),
            ValueType::I32(v) => format_value_type!(v, fmt, pre, post),
            ValueType::I64(v) => format_value_type!(v, fmt, pre, post),
            ValueType::None => Ok(Node::new("None".into())),
        };
        let mut node = node?;
        node.kind = NodeKind::Value(*self);
        Ok(node)
    }
}

#[cfg(test)]
mod test {
    use super::{arch::a6502, Address};
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

    #[test]
    fn a6502() {
        // byte and immediate
        test_arch_result(
            &a6502::ARCH,
            &[0xFF, 0xaa, 0x69, 0x02, 0x01],
            "00000000 .db $ff\n00000001 .db $aa\n00000002 adc #$02\n00000004 .db $01\n",
            0x5,
        );

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
    }
}
