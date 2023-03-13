pub mod command;

use crate::{
    core::dasm::arch::{Archs, Context},
    prelude::{Config, FdResult},
};
use rustyline::error::ReadlineError;

use self::command::{default_actions, Interactive};

pub enum CallbackKind {
    None,
}

pub trait InteractiveCallback = FnMut(&str, CallbackKind) -> FdResult<()>;

pub fn default_interactive_callback(s: &str, _kind: CallbackKind) -> FdResult<()> {
    print!("{}", s);
    Ok(())
}

pub fn command_line(cfg: &Config, arch: Archs, mut ctx: Context, data: Vec<u8>) -> FdResult<()> {
    let mut rl = rustyline::DefaultEditor::new().expect("Unable to init interactive mode");
    let actions = default_actions();
    let mut interactive = Interactive { actions, data };
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).expect("History error");
                if let Err(err) =
                    interactive.execute(default_interactive_callback, &line, &arch, &mut ctx, cfg)
                {
                    eprintln!("{:?}", err);
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => return Ok(()),
            Err(err) => eprintln!("{:?}", err),
        }
    }
}
