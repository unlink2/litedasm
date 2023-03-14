use std::{
    io::{BufReader, LineWriter, Read, Write},
    path::{Path, PathBuf},
};

use log::info;

use crate::{
    core::dasm::{
        arch::{Archs, Context, DisasCallback},
        Address,
    },
    prelude::{auto_radix_address, auto_radix_usize, ArchKind, Config, Error, FdResult},
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
            optional_file_read_path_parser,
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
        Action::new(
            "sc",
            vec![Param::new("path")],
            optional_ctx_write_path_parser,
            "Save the current context",
        ),
        Action::new(
            "sa",
            vec![Param::new("path")],
            optional_arch_write_path_parser,
            "Save the current architecture",
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
    ReadFile(Option<PathBuf>),
    ReadContext(Option<PathBuf>),
    ReadArch(Option<ArchKind>),
    SaveArch(Option<PathBuf>),
    SaveContext(Option<PathBuf>),
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
                let mut f = Self::open_input(path.as_deref())?;
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
            Commands::ReadContext(_) => todo!(),
            Commands::ReadArch(_) => todo!(),
            Commands::SaveArch(path) => {
                info!("Saving arch to {path:?}");
                let mut f = Self::open_output(path.as_deref())?;
                f.write_all(
                    ron::ser::to_string_pretty(arch, Default::default())
                        .unwrap()
                        .as_bytes(),
                )?;
                Ok(())
            }
            Commands::SaveContext(path) => {
                info!("Saving ctx to {path:?}");
                let mut f = Self::open_output(path.as_deref())?;
                f.write_all(
                    ron::ser::to_string_pretty(ctx, Default::default())
                        .unwrap()
                        .as_bytes(),
                )?;

                Ok(())
            }
        }
        // Ok(())
    }

    pub fn open_input(path: Option<&Path>) -> FdResult<Box<dyn Read>> {
        Ok(if let Some(path) = &path {
            Box::new(BufReader::new(std::fs::File::open(path)?))
        } else {
            Box::new(BufReader::new(std::io::stdin()))
        })
    }

    pub fn open_output(path: Option<&Path>) -> FdResult<Box<dyn Write>> {
        Ok(if let Some(path) = &path {
            Box::new(LineWriter::new(
                std::fs::File::options().write(true).open(path)?,
            ))
        } else {
            Box::new(LineWriter::new(std::io::stdout().lock()))
        })
    }
}

#[derive(Default)]
pub struct CommandContext {
    pub actions: ActionList,
    pub data: Vec<u8>,
}

impl CommandContext {
    pub fn from_reader(actions: ActionList, input: &mut dyn Read) -> FdResult<Self> {
        let mut buffer = Vec::new();
        input.read_to_end(&mut buffer)?;
        Ok(Self {
            actions,
            data: buffer,
        })
    }

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

fn try_get_arg(args: &[&str], params: &[Param], index: usize) -> FdResult<String> {
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

fn get_optional_arg(args: &[&str], params: &[Param], index: usize) -> Option<String> {
    try_get_arg(args, params, index).ok()
}

fn has_too_many_args(args: &[&str], params: &[Param]) -> FdResult<()> {
    if args.len() > params.len() {
        Err(Error::TooManyArguments)
    } else {
        Ok(())
    }
}

fn expand_path(path: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(path).into_owned())
}

fn help_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;

    let cmd = try_get_arg(args, params, 0)?;

    Ok(Commands::Help(cmd))
}

fn exit_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    Ok(Commands::Exit)
}

fn disas_start_label_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let label = try_get_arg(args, params, 0)?;
    // let to: usize = auto_radix_usize(&get_arg_or(args, params, 1)?)?;

    Ok(Commands::SetStartLabel(label))
}

fn disas_read_len_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let to: usize = auto_radix_usize(&try_get_arg(args, params, 0)?)?;
    Ok(Commands::SetReadLen(to))
}

fn disas_code_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    Ok(Commands::DisasCode)
}

fn optional_file_read_path_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let path = get_optional_arg(args, params, 0);

    if let Some(path) = path {
        Ok(Commands::SaveContext(Some(expand_path(&path))))
    } else {
        Ok(Commands::ReadFile(None))
    }
}

fn optional_ctx_write_path_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let path = get_optional_arg(args, params, 0);

    if let Some(path) = path {
        Ok(Commands::SaveContext(Some(expand_path(&path))))
    } else {
        Ok(Commands::SaveContext(None))
    }
}

fn optional_arch_write_path_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let path = get_optional_arg(args, params, 0);

    if let Some(path) = path {
        Ok(Commands::SaveArch(Some(expand_path(&path))))
    } else {
        Ok(Commands::SaveArch(None))
    }
}

fn disas_start_address_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let address = auto_radix_address(&try_get_arg(args, params, 0)?)?;

    Ok(Commands::SetStartAddress(address))
}
