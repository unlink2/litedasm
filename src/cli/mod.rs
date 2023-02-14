use crate::core::{
    config::{generate_completion, CFG},
    dasm::arch::{a6502, default_callback},
    error::FdResult,
};

pub fn init() -> FdResult<()> {
    if let Some(shell) = CFG.completions {
        generate_completion(shell);
        std::process::exit(0);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::core::dasm::arch::Arch;

    fn collect_arch(arch: &Arch, data: &[u8], expected: &str) {
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
        collect_arch(
            &a6502::ARCH,
            &[0xFF, 0xaa, 0x69, 0x02, 0x1],
            ".db ff\n.db aa\nadc #$02\n.db 01\n",
        );
    }
}
