use std::{
    io::{LineWriter, Read},
    path::PathBuf,
};

use log::info;

use crate::{
    cli::print_callback,
    core::dasm::arch::{Archs, Context},
    prelude::{auto_radix_usize, Config, Error, FdResult},
};

use super::{CallbackKind, InteractiveCallback};

pub fn default_actions() -> ActionList {
    let mut actions = vec![
        Action::new(
            "?",
            vec![Param::with_default("command", "")],
            help_parser,
            "Display help",
        ),
        Action::new("q", vec![], exit_parser, "Quit the program"),
        Action::new("dc", vec![], dump_code_parser, "Dump code"),
        Action::new(
            "rf",
            vec![Param::new("path")],
            read_file_parser,
            "Read a file",
        ),
        Action::new(
            "dsl",
            vec![Param::new("label")],
            dump_start_label_parser,
            "Set dump starting point to a label",
        ),
        Action::new(
            "drl",
            vec![Param::new("len")],
            dump_read_len_parser,
            "Set dump read lenght",
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
    DumpCode,
    SetStartLabel(String),
    SetReadLen(usize),
    ReadFile(PathBuf),
}

impl Commands {
    pub fn execute(
        &self,
        mut f: impl InteractiveCallback,
        arch: &Archs,
        ctx: &mut Context,
        interactive: &mut Interactive,
        cfg: &Config,
    ) -> FdResult<()> {
        match self {
            Commands::Exit => std::process::exit(0),
            Commands::Help(cmd) => interactive.actions.help(&mut f, &cmd),
            Commands::DumpCode => {
                let mut output = LineWriter::new(std::io::stdout().lock());
                ctx.restart();
                arch.disas_ctx(
                    |node, kind, data, arch, ctx| {
                        print_callback(node, kind, data, arch, ctx, &mut output, cfg)
                    },
                    &interactive.data,
                    ctx,
                )?;

                Ok(())
            }
            Commands::ReadFile(path) => {
                let mut f = std::fs::File::open(path)?;
                let mut buffer = Vec::new();
                f.read_to_end(&mut buffer)?;
                interactive.data = buffer;
                info!("Binary loaded from {:?}", path);
                Ok(())
            }
            Commands::SetStartLabel(label) => {
                ctx.set_start_to_symbol(label)?;
                info!("New ctx start address: {:x}", ctx.start_read);
                if let Some(last_len) = interactive.last_len {
                    ctx.set_len(last_len);
                    info!("New ctx end address: {:?}", ctx.end_read);
                }

                Ok(())
            }
            Commands::SetReadLen(len) => {
                ctx.set_len(*len);
                info!("New ctx end address: {:?}", ctx.end_read);
                interactive.last_len = Some(*len);
                Ok(())
            }
        }
        // Ok(())
    }
}

#[derive(Default)]
pub struct Interactive {
    pub actions: ActionList,
    pub data: Vec<u8>,

    pub last_len: Option<usize>,
}

impl Interactive {
    pub fn execute(
        &mut self,
        f: impl InteractiveCallback,
        input: &str,
        arch: &Archs,
        ctx: &mut Context,
        cfg: &Config,
    ) -> FdResult<()> {
        let cmd = self.actions.eval(input)?;
        cmd.execute(f, &arch, ctx, self, cfg)?;
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
fn dump_start_label_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let label = get_arg_or(args, params, 0)?;
    // let to: usize = auto_radix_usize(&get_arg_or(args, params, 1)?)?;

    Ok(Commands::SetStartLabel(label))
}

fn dump_read_len_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let to: usize = auto_radix_usize(&get_arg_or(args, params, 0)?)?;
    Ok(Commands::SetReadLen(to))
}

fn dump_code_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    Ok(Commands::DumpCode)
}

fn read_file_parser(args: &[&str], params: &[Param]) -> FdResult<Commands> {
    has_too_many_args(args, params)?;
    let path: PathBuf = get_arg_or_into(args, params, 0)?;

    Ok(Commands::ReadFile(path))
}
