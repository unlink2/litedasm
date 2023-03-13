use crate::{
    core::dasm::arch::{Archs, Context},
    prelude::{Error, FdResult},
};

use super::{CallbackKind, InteractiveCallback};

pub fn default_actions() -> ActionList {
    ActionList {
        actions: vec![
            Action::new(
                "?",
                vec![Param::with_default("command", "")],
                help_parser,
                "Display help",
            ),
            Action::new("q", vec![], exit_parser, "Quit the program"),
        ],
    }
}

/// Command syntax:
/// A list of actions, followed by a list of sub actions
/// followed by a list of parameters
/// the full command could look like this:
/// abc 123 456
pub struct ActionList {
    actions: Vec<Action>,
}

impl ActionList {
    pub fn eval(&self, input: &str) -> FdResult<Commands> {
        // tokenize the input
        let mut split = input.split_whitespace();
        let cmd = split.next().unwrap_or("");
        let args: Vec<&str> = split.collect();
        let action = self
            .actions
            .iter()
            .find(|x| x.name == cmd)
            .ok_or(Error::UnknownCommand(cmd.into()))?;

        action.eval(&args)
    }

    fn help(&self, f: &mut dyn InteractiveCallback, cmd: &str) -> FdResult<()> {
        let mut printed = false;
        for action in &self.actions {
            if action.name.starts_with(cmd) {
                printed = true;
                action.help(f)?;
            }
        }
        if printed {
            Ok(())
        } else {
            Err(Error::UnknownCommand(cmd.into()))
        }
    }
}

#[derive(Default)]
pub struct Param {
    name: String,
    default_value: Option<String>,
}

impl Param {
    fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            default_value: None,
        }
    }

    fn with_default(name: &str, default_value: &str) -> Self {
        Self {
            name: name.into(),
            default_value: Some(default_value.into()),
        }
    }
}

type CommandParser = fn(&[&str], &[Param]) -> FdResult<Commands>;

pub struct Action {
    help: String,
    name: String,
    params: Vec<Param>,
    parser: CommandParser,
}

impl Action {
    fn new(name: &str, params: Vec<Param>, parser: CommandParser, help: &str) -> Self {
        Self {
            name: name.into(),
            help: help.into(),
            params,
            parser,
        }
    }

    fn eval(&self, args: &[&str]) -> FdResult<Commands> {
        (self.parser)(args, &self.params)
    }

    fn help(&self, f: &mut dyn InteractiveCallback) -> FdResult<()> {
        f(&self.name, super::CallbackKind::None)?;
        self.params.iter().try_for_each(|x| {
            if let Some(default_value) = &x.default_value {
                f(
                    &format!(" [{}='{}']", x.name, default_value),
                    CallbackKind::None,
                )
            } else {
                f(&format!(" [{}]", x.name), CallbackKind::None)
            }
        })?;
        f(&format!(" {}\n", self.help), CallbackKind::None)?;
        Ok(())
    }
}

pub enum Commands {
    Exit,
    Help(String),
}

impl Commands {
    pub fn execute(
        &self,
        mut f: impl InteractiveCallback,
        _arch: &Archs,
        _ctx: &mut Context,
        _data: Option<&[u8]>,
        actions: &ActionList,
    ) -> FdResult<()> {
        match self {
            Commands::Exit => std::process::exit(0),
            Commands::Help(cmd) => actions.help(&mut f, &cmd),
        }
        // Ok(())
    }
}

/* Command parsers */

fn get_arg_or(args: &[&str], params: &[Param], index: usize) -> FdResult<String> {
    let arg = args.get(index);
    let param = params.get(index);

    if let Some(arg) = arg && let Some(_param) = param {
        Ok(arg.to_string())
    } else if let Some(param) = param {
        if let Some(def) = &param.default_value {
            Ok(def.into())
        } else {
            Err(Error::InsufficientArguments)
        }
    } else {
        Err(Error::InsufficientArguments)
    }
}

fn has_too_many_args(args: &[&str], params: &[Param]) -> FdResult<()> {
    if args.len() > params.len() {
        Err(Error::TooManyArguments)
    } else {
        Ok(())
    }
}

fn help_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;

    let cmd = get_arg_or(args, params, 0)?;

    Ok(Commands::Help(cmd))
}

fn exit_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    Ok(Commands::Exit)
}
