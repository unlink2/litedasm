use crate::{
    core::{
        config::{generate_completion, CFG},
        dasm::arch::{a6502, default_callback},
        error::FdResult,
    },
    prelude::Config,
};

pub fn init(cfg: &Config) -> FdResult<()> {
    if let Some(shell) = CFG.completions {
        generate_completion(shell);
        std::process::exit(0);
    }

    // first get the arch
    let arch = cfg.arch.to_arch(cfg)?;
    // set up io
    let mut input = cfg.input()?;
    let mut output = cfg.output()?;

    // read all the input data into a buffer
    // FIXME this may be bad for larger files!
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    // first pass - generate symbols
    arch.disas(|_node, _data, _arch, _ctx| Ok(()), &buffer)?;

    // second pass - the actual output
    arch.disas(
        |node, _data, _arch, _ctx| {
            write!(output, "{}", node.string)?;
            Ok(())
        },
        &buffer,
    )?;

    Ok(())
}
