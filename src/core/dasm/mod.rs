use crate::prelude::{Error, FdResult};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use self::arch::{Node, NodeKind};

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
#[derive(Clone, Copy, Debug)]
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
    ($val: expr, $fmt:expr) => {
        match $fmt {
            ValueTypeFmt::Binary(width) => Ok(Node::new(format!("{:0width$b}", $val))),
            ValueTypeFmt::Decimal(width) => Ok(Node::new(format!("{:0width$}", $val))),
            ValueTypeFmt::LowerHex(width) => Ok(Node::new(format!("{:0width$x}", $val))),
            ValueTypeFmt::Octal(width) => Ok(Node::new(format!("{:0width$o}", $val))),
            ValueTypeFmt::UpperHex(width) => Ok(Node::new(format!("{:0width$X}", $val))),
            // _ => Err(Error::UnsupportedFormat($fmt)),
        }
    };
}

impl ValueType {
    pub fn try_to_node(&self, fmt: ValueTypeFmt) -> FdResult<Node> {
        let node: FdResult<Node> = match self {
            ValueType::U8(v) => format_value_type!(v, fmt),
            ValueType::U16(v) => format_value_type!(v, fmt),
            ValueType::U32(_) => todo!(),
            ValueType::U64(_) => todo!(),
            ValueType::I8(_) => todo!(),
            ValueType::I16(_) => todo!(),
            ValueType::I32(_) => todo!(),
            ValueType::I64(_) => todo!(),
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
            "00000000 .db ff\n00000001 .db aa\n00000002 adc #$02\n00000004 .db 01\n",
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
    }
}
