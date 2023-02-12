use crate::core::{
    config::{generate_completion, CFG},
    dasm::arch::{default_callback, Context},
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
    ctx.disas(default_callback, &[0], &mut stdout)
        .expect("Test code failed");

    Ok(())
}
