pub mod interactive;

use crate::{
    core::{
        config::generate_completion,
        dasm::{
            arch::{Arch, Archs, CallbackKind, Context, Node},
            Address,
        },
        error::FdResult,
    },
    prelude::{Config, DefSym, DisasCommand},
};
use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use std::{
    io::{prelude::*, LineWriter},
    path::PathBuf,
};

use self::interactive::{
    command::{default_actions, CommandContext},
    default_interactive_callback,
};

const CTX_DEFAULT_FILE: &str = "./ctx.ron";
const CTX_DEFAULT_FILE_VAR: &str = "LITEDASM_CTX_PATH";

fn get_ctx_file(cfg: &Config) -> Option<PathBuf> {
    if let Some(path) = &cfg.ctx_file {
        return Some(path.to_owned());
    }
    let path: PathBuf = match std::env::var(CTX_DEFAULT_FILE_VAR) {
        Ok(val) => val.into(),
        Err(_) => CTX_DEFAULT_FILE.into(),
    };

    if !path.exists() {
        None
    } else {
        Some(path)
    }
}

pub fn read_ctx(cfg: &Config) -> FdResult<Context> {
    let mut ctx = if let Some(path) = get_ctx_file(cfg) {
        info!(
            "Reading from context path '{}'",
            path.to_str().unwrap_or("")
        );
        let data = std::fs::read_to_string(path)?;
        ron::from_str(&data).expect("Unable to read context file")
    } else {
        info!("Using default context");
        Context::default()
    };

    if let Some(org) = cfg.org {
        ctx.org = org;
    }
    ctx.set_start(cfg.start_read);
    if let Some(label) = &cfg.start_at_label {
        ctx.set_start_to_symbol(label)?;
    }
    // ctx.set_org(ctx.org + ctx.start_read as Address);

    // ctx.set_end(cfg.end_read);
    ctx.set_len(cfg.read_len);

    Ok(ctx)
}

fn verbose_to_level_filter(v: u8) -> LevelFilter {
    match v {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

pub fn write_ctx(cfg: &Config, ctx: &Context) -> FdResult<()> {
    let data =
        ron::ser::to_string_pretty(ctx, Default::default()).expect("Unable to convert context");

    if let Some(path) = get_ctx_file(cfg) {
        info!("Writing context to '{}'", path.to_str().unwrap_or(""));
        let mut f = std::fs::File::create(path)?;
        f.write_all(&data.into_bytes())?;
    } else {
        println!("{data}");
    }
    Ok(())
}

pub fn init(cfg: &Config) -> FdResult<()> {
    SimpleLogger::new()
        .with_level(verbose_to_level_filter(cfg.verbose))
        .init()
        .expect("Failed initializing logger");

    if let Some(shell) = cfg.completions {
        generate_completion(shell);
        std::process::exit(0);
    }

    // first get the arch
    let mut arch = cfg.arch.to_arch(cfg)?;
    let mut ctx = read_ctx(cfg)?;

    // run commands using the parser
    {
        let mut interactive = CommandContext {
            actions: default_actions(),
            data: Default::default(),
            ..Default::default()
        };
        let mut output = LineWriter::new(std::io::stdout().lock());
        for run in &cfg.run {
            interactive.execute(
                default_interactive_callback,
                |node, kind, data, arch, ctx| {
                    print_callback(node, kind, data, arch, ctx, &mut output, cfg)
                },
                run,
                &mut arch,
                &mut ctx,
                cfg,
            )?;
        }
    }

    if let Some(command) = &cfg.command {
        match command {
            crate::prelude::Commands::Org { address } => org(cfg, *address, &arch, &mut ctx),
            crate::prelude::Commands::Disas(d) => disas(cfg, d, &arch, &mut ctx),
            crate::prelude::Commands::DumpArch => dump_arch(cfg, &arch),
            crate::prelude::Commands::DumpCtx => dump_ctx(cfg, &ctx),
            crate::prelude::Commands::DefSym(ds) => defsym(cfg, ds, &arch, &mut ctx),
            crate::prelude::Commands::Patch(d) => patch(cfg, d, &mut ctx),
            crate::prelude::Commands::Interactive { input } => {
                let mut f = std::fs::File::open(input)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                interactive::command_line(cfg, arch, ctx, buffer)
            }
        }
    } else {
        interactive::command_line(cfg, arch, ctx, vec![])
    }
}

fn patch(_cfg: &Config, disas: &DisasCommand, ctx: &mut Context) -> FdResult<()> {
    // set up io
    let mut input = disas.input()?;
    let mut output = disas.output()?;

    // read all the input data into a buffer
    // FIXME this may be bad for larger files!
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;

    let res = ctx.patch(&buffer)?;
    output.write_all(&res)?;

    Ok(())
}

fn dump_arch(_cfg: &Config, arch: &Archs) -> FdResult<()> {
    println!(
        "{}",
        ron::ser::to_string_pretty(&arch, Default::default()).unwrap()
    );
    Ok(())
}

fn dump_ctx(_cfg: &Config, ctx: &Context) -> FdResult<()> {
    println!(
        "{}",
        ron::ser::to_string_pretty(&ctx, Default::default()).unwrap()
    );
    Ok(())
}

fn org(cfg: &Config, address: Address, _arch: &Archs, ctx: &mut Context) -> FdResult<()> {
    ctx.org = address;
    write_ctx(cfg, ctx)
}

fn print_callback<T>(
    node: &Node,
    kind: CallbackKind,
    _raw: &[u8],
    _arch: &Arch,
    ctx: &mut Context,
    output: &mut T,
    cfg: &Config,
) -> FdResult<()>
where
    T: Write,
{
    if let CallbackKind::Pad(n) = kind {
        if n > ctx.tr_ctx.line_len {
            // wirte pads
            for _ in 0..n - ctx.tr_ctx.line_len {
                write!(output, " ")?;
            }
        }
    }

    if cfg.no_color {
        write!(output, "{}", node.string)?;
    } else {
        use console::style;
        let s = &node.string;

        write!(
            output,
            "{}",
            match kind {
                CallbackKind::Val => style(s).cyan(),
                CallbackKind::Raw => style(s).red(),
                CallbackKind::Address => style(s).yellow(),
                CallbackKind::Label => style(s).green(),
                CallbackKind::Symbol => style(s).cyan(),
                CallbackKind::MatcherName => style(s),
                CallbackKind::Static => style(s),
                _ => style(s),
            }
        )?;
    }
    Ok(())
}

fn disas(cfg: &Config, disas: &DisasCommand, arch: &Archs, ctx: &mut Context) -> FdResult<()> {
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
        arch.disas_ctx(|_node, _kind, _data, _arch, _ctx| Ok(()), &buffer, ctx)?;
        ctx.restart();
        ctx.analyze = false;
    }

    // second pass - the actual output
    arch.disas_ctx(
        |node, kind, data, arch, ctx| print_callback(node, kind, data, arch, ctx, &mut output, cfg),
        &buffer,
        ctx,
    )?;
    Ok(())
}

fn defsym(cfg: &Config, defsym: &DefSym, _arch: &Archs, ctx: &mut Context) -> FdResult<()> {
    ctx.def_symbol(defsym.clone().into());
    write_ctx(cfg, ctx)
}
