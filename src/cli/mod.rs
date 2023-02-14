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

    let test_arch = &a6502::ARCH;
    test_arch.disas(default_callback, &[0xFF]).unwrap();

    Ok(())
}
