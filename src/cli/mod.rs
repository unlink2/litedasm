use crate::core::{
    config::{generate_completion, CFG},
    error::FdResult,
};

pub fn init() -> FdResult<()> {
    if let Some(shell) = CFG.completions {
        generate_completion(shell);
        std::process::exit(0);
    }
    Ok(())
}
