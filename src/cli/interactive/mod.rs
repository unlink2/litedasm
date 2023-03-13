pub mod command;

use crate::{
    core::dasm::arch::{Archs, Context},
    prelude::{Config, FdResult},
};
use rustyline::error::ReadlineError;

use self::command::default_actions;

pub fn command_line(
    _cfg: &Config,
    arch: Archs,
    mut ctx: Context,
    data: Option<Vec<u8>>,
) -> FdResult<()> {
    let mut rl = rustyline::DefaultEditor::new().expect("Unable to init interactive mode");
    let mut stdout = std::io::stdout();
    let actions = default_actions();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).expect("History error");
                let cmd = actions.eval(&line);
                match cmd {
                    Ok(cmd) => {
                        if let Err(err) =
                            cmd.execute(&mut stdout, &arch, &mut ctx, data.as_deref(), &actions)
                        {
                            eprintln!("{:?}", err);
                        }
                    }
                    Err(err) => eprintln!("{:?}", err),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => return Ok(()),
            Err(err) => eprintln!("{:?}", err),
        }
    }
}
