pub mod command;

use std::io::LineWriter;

use crate::{
    core::dasm::arch::{Archs, Context},
    prelude::{Config, FdResult},
};
use rustyline::error::ReadlineError;

use self::command::{default_actions, CommandContext};

use super::print_callback;

pub enum CallbackKind {
    None,
}

pub trait CommandCallback = FnMut(&str, CallbackKind) -> FdResult<()>;

pub fn default_interactive_callback(s: &str, _kind: CallbackKind) -> FdResult<()> {
    print!("{}", s);
    Ok(())
}

pub fn command_line(
    cfg: &Config,
    mut arch: Archs,
    mut ctx: Context,
    data: Vec<u8>,
) -> FdResult<()> {
    let mut rl = rustyline::DefaultEditor::new().expect("Unable to init interactive mode");
    let actions = default_actions();
    let mut cmd_ctx = CommandContext {
        actions,
        data,
        ..Default::default()
    };
    let mut output = LineWriter::new(std::io::stdout().lock());
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if !line.is_empty() {
                    rl.add_history_entry(line.as_str()).expect("History error");
                    if let Err(err) = cmd_ctx.execute(
                        default_interactive_callback,
                        |node, kind, data, arch, ctx| {
                            print_callback(node, kind, data, arch, ctx, &mut output, cfg)
                        },
                        &line,
                        &mut arch,
                        &mut ctx,
                        cfg,
                    ) {
                        eprintln!("{:?}", err);
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => return Ok(()),
            Err(err) => eprintln!("{:?}", err),
        }
    }
}
