use crate::{
    core::dasm::arch::{Archs, Context},
    prelude::{Config, FdResult},
};
use rustyline::error::ReadlineError;

enum Commands {
    Exit,
}

impl Commands {
    pub fn execute(&self, _arch: &Archs, _ctx: &mut Context, _data: Option<&[u8]>) -> FdResult<()> {
        match self {
            Commands::Exit => std::process::exit(0),
        }
        // Ok(())
    }
}

impl TryFrom<String> for Commands {
    type Error = crate::prelude::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "exit" => Ok(Self::Exit),
            _ => Err(crate::prelude::Error::UnknownCommand(value)),
        }
    }
}

pub fn command_line(
    _cfg: &Config,
    arch: Archs,
    mut ctx: Context,
    data: Option<Vec<u8>>,
) -> FdResult<()> {
    let mut rl = rustyline::DefaultEditor::new().expect("Unable to init interactive mode");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let cmd = Commands::try_from(line);
                match cmd {
                    Ok(cmd) => cmd.execute(&arch, &mut ctx, data.as_deref())?,
                    Err(err) => eprintln!("{:?}", err),
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => return Ok(()),
            Err(err) => eprintln!("{:?}", err),
        }
    }
}
