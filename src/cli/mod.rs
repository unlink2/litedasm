use crate::core::{
    config::{generate_completion, CFG},
    dasm::arch::{default_callback, ArchDef, Context},
    error::FdResult,
};

pub fn init() -> FdResult<()> {
    if let Some(shell) = CFG.completions {
        generate_completion(shell);
        std::process::exit(0);
    }

    // TODO remove test code
    let mut stdout = std::io::stdout().lock();
    let mut ctx = Context::default();
    let arch = ArchDef::default();
    ctx.disas(
        |a, b, c, d| default_callback(a, b, c, d),
        &[0],
        &arch,
        &mut stdout,
    )
    .expect("Test code failed");

    Ok(())
}
