use std::{fs::File, io::Read, path::PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::prelude::FdResult;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone)]
pub enum Patch {
    Data { offset: usize, data: Vec<u8> },
    Rep { offset: usize, byte: u8, len: usize },
    File { offset: usize, path: PathBuf },
}

impl Patch {
    pub fn apply(&self, data: &mut Vec<u8>) -> FdResult<()> {
        let (offset, patch) = match self {
            Self::Data { offset, data } => (*offset, data.to_vec()),
            Self::Rep { offset, byte, len } => (*offset, vec![*byte; *len]),
            Self::File { offset, path } => {
                let mut f = File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                (*offset, buffer)
            }
        };
        for (i, b) in patch.iter().enumerate() {
            let index = i + offset;
            if data.len() <= index {
                data.push(*b)
            } else {
                data[index] = *b;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::core::dasm::arch::Context;

    use super::Patch;

    #[test]
    fn data() {
        let mut ctx = Context::default();
        ctx.patches.push(Patch::Data {
            offset: 1,
            data: vec![0, 1, 2, 3],
        });

        let test_data = vec![0, 1, 2, 3];
        let res = ctx.patch(&test_data).unwrap();

        assert_eq!(vec![0, 0, 1, 2, 3], res);
    }

    #[test]
    fn repeat() {
        let mut ctx = Context::default();
        ctx.patches.push(Patch::Rep {
            offset: 1,
            byte: 5,
            len: 4,
        });

        let test_data = vec![0, 1, 2, 3];
        let res = ctx.patch(&test_data).unwrap();

        assert_eq!(vec![0, 5, 5, 5, 5], res);
    }
}
