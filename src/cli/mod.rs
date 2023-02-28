use crate::{
    core::{config::generate_completion, dasm::arch::Context, error::FdResult},
    prelude::Config,
};

pub fn ctx(cfg: &Config) -> FdResult<Context> {
    if let Some(path) = &cfg.ctx_file {
        let data = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&data).expect("Unable to read context file"))
    } else {
        Ok(Context::default())
    }
}

pub fn init(cfg: &Config) -> FdResult<()> {
    if let Some(shell) = cfg.completions {
        generate_completion(shell);
        std::process::exit(0);
    }

    // first get the arch
    let arch = cfg.arch.to_arch(cfg)?;

    // dump mode?
    if cfg.dump_arch {
        println!("{}", serde_json::to_string_pretty(&arch).unwrap());
        std::process::exit(0);
    }

    let mut ctx = ctx(cfg)?;
    if cfg.dump_ctx {
        println!("{}", serde_json::to_string_pretty(&ctx).unwrap());
        std::process::exit(0);
    }

    // set up io
    let mut input = cfg.input()?;
    let mut output = cfg.output()?;

    // read all the input data into a buffer
    // FIXME this may be bad for larger files!
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    // first pass - generate symbols
    if cfg.pre_analyze {
        ctx.analyze = true;
        arch.disas_ctx(|_node, _data, _arch, _ctx| Ok(()), &buffer, &mut ctx)?;
        ctx.restart();
        ctx.analyze = false;
    }

    // second pass - the actual output
    arch.disas_ctx(
        |node, _data, _arch, _ctx| {
            write!(output, "{}", node.string)?;
            Ok(())
        },
        &buffer,
        &mut ctx,
    )?;

    Ok(())
}
