use crate::prelude::{Error, FdResult};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

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
            ValueTypeFmt::Binary(width) => Ok(format!("{:0width$b}", $val)),
            ValueTypeFmt::Decimal(width) => Ok(format!("{:0width$}", $val)),
            ValueTypeFmt::LowerHex(width) => Ok(format!("{:0width$x}", $val)),
            ValueTypeFmt::Octal(width) => Ok(format!("{:0width$o}", $val)),
            ValueTypeFmt::UpperHex(width) => Ok(format!("{:0width$X}", $val)),
            _ => Err(Error::UnsupportedFormat($fmt)),
        }
    };
}

impl ValueType {
    pub fn try_to_string(&self, fmt: ValueTypeFmt) -> FdResult<String> {
        match self {
            ValueType::U8(v) => format_value_type!(v, fmt),
            ValueType::U16(_) => todo!(),
            ValueType::U32(_) => todo!(),
            ValueType::U64(_) => todo!(),
            ValueType::I8(_) => todo!(),
            ValueType::I16(_) => todo!(),
            ValueType::I32(_) => todo!(),
            ValueType::I64(_) => todo!(),
            ValueType::None => Ok("None".into()),
        }
    }
}

#[cfg(test)]
mod test {
    use super::arch::a6502;
    use crate::core::dasm::arch::Arch;

    fn test_arch_result(arch: &Arch, data: &[u8], expected: &str) {
        let mut result = "".to_string();
        arch.disas(
            |s, _arch, _ctx| {
                result.push_str(s);
                Ok(())
            },
            data,
        )
        .unwrap();

        assert_eq!(expected, result);
    }

    #[test]
    fn a6502() {
        test_arch_result(
            &a6502::ARCH,
            &[0xFF, 0xaa, 0x69, 0x02, 0x1],
            ".db ff\n.db aa\nadc #$02\n.db 01\n",
        );

        test_arch_result(&a6502::ARCH, &[0x75, 0x12], "adc $12, x\n");
    }
}
