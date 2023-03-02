use crate::{
    core::{
        config::generate_completion,
        dasm::{
            arch::{Archs, Context},
            Address,
        },
        error::FdResult,
    },
    prelude::{Config, DisasCommand},
};
use std::io::prelude::*;

pub fn read_ctx(cfg: &Config) -> FdResult<Context> {
    let mut ctx = if let Some(path) = &cfg.ctx_file {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data).expect("Unable to read context file")
    } else {
        Context::default()
    };
    ctx.start_read = cfg.start_read.unwrap_or(0);
    ctx.end_read = cfg.end_read;
    Ok(ctx)
}

pub fn write_ctx(cfg: &Config, ctx: &Context) -> FdResult<()> {
    let data = serde_json::to_string_pretty(ctx).expect("Unable to convert context");

    if let Some(path) = &cfg.ctx_file {
        let mut f = std::fs::File::create(path)?;
        f.write_all(&data.into_bytes())?;
    } else {
        println!("{}", data);
    }
    Ok(())
}

pub fn init(cfg: &Config) -> FdResult<()> {
    if let Some(shell) = cfg.completions {
        generate_completion(shell);
        std::process::exit(0);
    }

    // first get the arch
    let arch = cfg.arch.to_arch(cfg)?;
    let mut ctx = read_ctx(cfg)?;

    match &cfg.command {
        crate::prelude::Commands::CtxOrg { address } => org(cfg, *address, &arch, &mut ctx),
        crate::prelude::Commands::Disas(d) => disas(cfg, d, &arch, &mut ctx),
        crate::prelude::Commands::DumpArch => dump_arch(cfg, &arch),
        crate::prelude::Commands::DumpCtx => dump_ctx(cfg, &ctx),
    }
}

fn dump_arch(_cfg: &Config, arch: &Archs) -> FdResult<()> {
    println!("{}", serde_json::to_string_pretty(&arch).unwrap());
    Ok(())
}

fn dump_ctx(_cfg: &Config, ctx: &Context) -> FdResult<()> {
    println!("{}", serde_json::to_string_pretty(&ctx).unwrap());
    Ok(())
}

fn org(cfg: &Config, address: Address, _arch: &Archs, ctx: &mut Context) -> FdResult<()> {
    ctx.org = address;
    write_ctx(cfg, ctx)
}

fn disas(_cfg: &Config, disas: &DisasCommand, arch: &Archs, ctx: &mut Context) -> FdResult<()> {
    // set up io
    let mut input = disas.input()?;
    let mut output = disas.output()?;

    // read all the input data into a buffer
    // FIXME this may be bad for larger files!
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    // first pass - generate symbols
    if disas.pre_analyze {
        ctx.analyze = true;
        arch.disas_ctx(|_node, _data, _arch, _ctx| Ok(()), &buffer, ctx)?;
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
        ctx,
    )?;
    Ok(())
}
