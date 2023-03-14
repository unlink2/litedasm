use std::{io::Read, path::PathBuf};

use log::info;

use crate::{
    core::dasm::{
        arch::{Archs, Context, DisasCallback},
        Address,
    },
    prelude::{auto_radix_address, auto_radix_usize, Config, Error, FdResult},
};

use super::{CallbackKind, CommandCallback};

pub fn default_actions() -> ActionList {
    let mut actions = vec![
        Action::new(
            "?",
            vec![Param::with_default("command", "")],
            help_parser,
            "Display help",
        ),
        Action::new("q", vec![], exit_parser, "Quit the program"),
        Action::new("dc", vec![], disas_code_parser, "Disassemble code"),
        Action::new(
            "rf",
            vec![Param::new("path")],
            read_file_parser,
            "Read a file",
        ),
        Action::new(
            "dcl",
            vec![Param::new("label")],
            disas_start_label_parser,
            "Set disas starting point to a label",
        ),
        Action::new(
            "dcr",
            vec![Param::new("len")],
            disas_read_len_parser,
            "Set disas read length",
        ),
        Action::new(
            "dca",
            vec![Param::new("address")],
            disas_start_address_parser,
            "Set disas starting point to an address",
        ),
    ];

    actions.sort_by_key(|l| l.name.to_owned());
    ActionList { actions }
}

/// Command syntax:
/// A list of actions, followed by a list of sub actions
/// followed by a list of parameters
/// the full command could look like this:
/// abc 123 456
#[derive(Default)]
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

    fn help(&self, f: &mut dyn CommandCallback, cmd: &str) -> FdResult<()> {
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

    fn help(&self, f: &mut dyn CommandCallback) -> FdResult<()> {
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
    DisasCode,
    SetStartLabel(String),
    SetStartAddress(Address),
    SetReadLen(usize),
    ReadFile(PathBuf),
}

impl Commands {
    pub fn execute(
        &self,
        mut f: impl CommandCallback,
        mut dcb: impl DisasCallback,
        arch: &Archs,
        ctx: &mut Context,
        cmd_ctx: &mut CommandContext,
        _cfg: &Config,
    ) -> FdResult<()> {
        match self {
            Commands::Exit => std::process::exit(0),
            Commands::Help(cmd) => cmd_ctx.actions.help(&mut f, &cmd),
            Commands::DisasCode => {
                ctx.restart();
                arch.disas_ctx(&mut dcb, &cmd_ctx.data, ctx)?;

                Ok(())
            }
            Commands::ReadFile(path) => {
                let mut f = std::fs::File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                cmd_ctx.data = buffer;
                info!("Binary loaded from {:?}", path);
                Ok(())
            }
            Commands::SetStartLabel(label) => {
                ctx.set_start_to_symbol(label)?;
                info!("New ctx start address: {:x}", ctx.start_read);

                Ok(())
            }
            Commands::SetReadLen(len) => {
                ctx.set_len(Some(*len));
                info!("New ctx read len: {:?}", ctx.len_read);
                Ok(())
            }
            Commands::SetStartAddress(address) => {
                ctx.set_start(Some(*address as usize));
                info!("New ctx start address: {:x}", ctx.start_read);

                Ok(())
            }
        }
        // Ok(())
    }
}

#[derive(Default)]
pub struct CommandContext {
    pub actions: ActionList,
    pub data: Vec<u8>,
}

impl CommandContext {
    pub fn execute(
        &mut self,
        f: impl CommandCallback,
        dcb: impl DisasCallback,
        input: &str,
        arch: &Archs,
        ctx: &mut Context,
        cfg: &Config,
    ) -> FdResult<()> {
        let cmd = self.actions.eval(input)?;
        cmd.execute(f, dcb, &arch, ctx, self, cfg)?;
        Ok(())
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

fn get_arg_or_into<T>(args: &[&str], params: &[Param], index: usize) -> FdResult<T>
where
    T: From<String>,
{
    Ok(get_arg_or(args, params, index)?.into())
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
fn disas_start_label_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let label = get_arg_or(args, params, 0)?;
    // let to: usize = auto_radix_usize(&get_arg_or(args, params, 1)?)?;

    Ok(Commands::SetStartLabel(label))
}

fn disas_read_len_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let to: usize = auto_radix_usize(&get_arg_or(args, params, 0)?)?;
    Ok(Commands::SetReadLen(to))
}

fn disas_code_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    Ok(Commands::DisasCode)
}

fn read_file_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let path: PathBuf = get_arg_or_into(args, params, 0)?;

    Ok(Commands::ReadFile(path))
}

fn disas_start_address_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let address = auto_radix_address(&get_arg_or(args, params, 0)?)?;

    Ok(Commands::SetStartAddress(address))
}
