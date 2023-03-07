use std::{
    fmt::Display,
    io::{BufReader, LineWriter, Read, Write},
    num::ParseIntError,
    path::PathBuf,
};

use super::dasm::{
    arch::{a6502, a65c02, a65c816, Archs},
    symbols::{Scope, Symbol, SymbolKind},
    Address, ValueType,
};
use crate::prelude::FdResult;
#[cfg(feature = "cli")]
use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
#[cfg(feature = "cli")]
use clap_complete::{generate, Generator, Shell};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CFG: Config = Config::new();
}

#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default)]
pub struct DisasCommand {
    input: Option<PathBuf>,
    output: Option<PathBuf>,

    #[cfg_attr(feature = "cli", arg(long, short))]
    pub pre_analyze: bool,
}

impl DisasCommand {
    pub fn input(&self) -> FdResult<Box<dyn Read>> {
        Ok(if let Some(path) = &self.input {
            Box::new(BufReader::new(std::fs::File::open(path)?))
        } else {
            Box::new(BufReader::new(std::io::stdin()))
        })
    }

    pub fn output(&self) -> FdResult<Box<dyn Write>> {
        Ok(if let Some(path) = &self.output {
            Box::new(LineWriter::new(
                std::fs::File::options().write(true).open(path)?,
            ))
        } else {
            Box::new(LineWriter::new(std::io::stdout().lock()))
        })
    }
}

#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default)]
pub struct DefSym {
    #[cfg_attr(feature = "cli", clap(long, short, value_parser = auto_radix_address))]
    from: Option<Address>,
    #[cfg_attr(feature = "cli", clap(long, short, value_parser = auto_radix_address))]
    to: Option<Address>,
    #[cfg_attr(feature = "cli", clap(long, short))]
    len: Option<usize>,
    #[cfg_attr(feature = "cli", clap(long, short))]
    const_value: bool,
    name: String,
    #[cfg_attr(feature = "cli", clap(value_parser = auto_radix_value))]
    pub value: ValueType,
}

#[allow(clippy::from_over_into)]
impl Into<Symbol> for DefSym {
    fn into(self) -> Symbol {
        Symbol {
            name: self.name.to_owned(),
            kind: if self.const_value {
                SymbolKind::Const
            } else {
                SymbolKind::Label
            },
            scope: if self.from.is_none() && self.to.is_none() {
                Scope::Global
            } else {
                Scope::Range(
                    self.from.unwrap_or(self.to.unwrap_or(0)),
                    self.to.unwrap_or(self.from.unwrap_or(0)),
                )
            },
            value: self.value,
            additional_values: Default::default(),
            len: self.len.unwrap_or(1),
        }
    }
}

#[cfg_attr(feature = "cli", derive(Subcommand))]
#[derive(Clone, Debug)]
pub enum Commands {
    Org {
        #[cfg_attr(feature = "cli", arg(value_parser = auto_radix_address))]
        address: Address,
    },
    DefSym(DefSym),
    Disas(DisasCommand),
    Patch(DisasCommand),
    DumpArch,
    DumpCtx,
}

impl Default for Commands {
    fn default() -> Self {
        Self::Disas(Default::default())
    }
}

#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[derive(Default, Copy, Clone, Debug)]
pub enum ArchKind {
    #[default]
    Arch6502,
    Arch65c02,
    Arch65c816,
    ArchCustom,
}

impl ArchKind {
    pub fn to_arch(&self, cfg: &Config) -> FdResult<Archs> {
        Ok(match self {
            ArchKind::Arch6502 => a6502::ARCH.to_owned(),
            ArchKind::Arch65c02 => a65c02::ARCH.to_owned(),
            ArchKind::Arch65c816 => a65c816::ARCH.to_owned(),
            #[cfg(feature = "serde")]
            ArchKind::ArchCustom => ron::from_str(&std::fs::read_to_string(
                cfg.arch_file.as_ref().expect("No arch file found"),
            )?)
            .expect("Error parsing arch file"),
            #[cfg(not(feature = "serde"))]
            ArchKind::ArchCustom => panic!("Custom architectures require serde feature"),
        })
    }
}

impl Display for ArchKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchKind::Arch6502 => write!(f, "arch6502"),
            ArchKind::ArchCustom => write!(f, "archCustom"),
            ArchKind::Arch65c02 => write!(f, "arch65c02"),
            ArchKind::Arch65c816 => write!(f, "arch65c816"),
        }
    }
}

fn auto_radix_value(s: &str) -> Result<ValueType, ParseIntError> {
    if s.starts_with("0x") {
        let s = &s[2..];
        ValueType::from_str_radix(s, 16)
    } else if s.starts_with("0b") {
        let s = &s[2..];
        ValueType::from_str_radix(s, 2)
    } else if s.starts_with("0o") {
        let s = &s[2..];
        ValueType::from_str_radix(s, 8)
    } else {
        ValueType::from_str_radix(s, 10)
    }
}

fn auto_radix_address(s: &str) -> Result<Address, ParseIntError> {
    if s.starts_with("0x") {
        let s = &s[2..];
        Address::from_str_radix(s, 16)
    } else if s.starts_with("0b") {
        let s = &s[2..];
        Address::from_str_radix(s, 2)
    } else if s.starts_with("0o") {
        let s = &s[2..];
        Address::from_str_radix(s, 8)
    } else {
        Address::from_str_radix(s, 10)
    }
}

fn auto_radix_usize(s: &str) -> Result<usize, ParseIntError> {
    if s.starts_with("0x") {
        let s = &s[2..];
        usize::from_str_radix(s, 16)
    } else if s.starts_with("0b") {
        let s = &s[2..];
        usize::from_str_radix(s, 2)
    } else if s.starts_with("0o") {
        let s = &s[2..];
        usize::from_str_radix(s, 8)
    } else {
        usize::from_str_radix(s, 10)
    }
}

#[derive(Debug, Default)]
#[cfg_attr(feature = "cli", derive(Parser))]
#[cfg_attr(feature = "cli", command(author, version, about, long_about = None))]
pub struct Config {
    #[cfg_attr(feature = "cli", command(subcommand))]
    pub command: Commands,

    // built in arch that may be loaded
    #[cfg_attr(feature = "cli", clap(long, short))]
    #[cfg_attr(feature = "cli", arg(default_value_t))]
    pub arch: ArchKind,

    // custom arch config file to load
    #[cfg_attr(feature = "cli", clap(long))]
    pub arch_file: Option<PathBuf>,

    #[cfg_attr(feature = "cli", clap(long, short))]
    pub ctx_file: Option<PathBuf>,

    #[cfg_attr(feature = "cli", arg(short, long, action = clap::ArgAction::Count))]
    pub verbose: u8,

    #[cfg_attr(feature = "cli", arg(long, short, value_parser = auto_radix_usize))]
    pub start_read: Option<usize>,

    #[cfg_attr(feature = "cli", arg(long, short, value_parser = auto_radix_usize))]
    pub end_read: Option<usize>,

    #[cfg_attr(feature = "cli", arg(long, short, value_parser = auto_radix_usize))]
    pub read_len: Option<usize>,

    #[cfg_attr(feature = "cli", arg(long, value_parser = auto_radix_address))]
    pub org: Option<Address>,

    #[cfg_attr(feature = "cli", arg(long))]
    pub no_color: bool,

    #[cfg_attr(feature = "cli", clap(long, value_name = "SHELL"))]
    #[cfg(feature = "cli")]
    pub completions: Option<Shell>,
}

impl Config {
    #[cfg(feature = "cli")]
    pub fn new() -> Self {
        Self::parse()
    }

    #[cfg(not(feature = "cli"))]
    pub fn new() -> Self {
        Default::default()
    }
}

#[cfg(feature = "cli")]
pub fn generate_completion<G: Generator>(gen: G) {
    generate(
        gen,
        &mut Config::command(),
        Config::command().get_name(),
        &mut std::io::stdout(),
    );
}
