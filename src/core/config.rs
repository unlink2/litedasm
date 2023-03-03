use std::{
    fmt::Display,
    io::{BufReader, LineWriter, Read, Write},
    path::PathBuf,
};

use super::dasm::{
    arch::{a6502, a65c02, a65c816, Archs},
    symbols::{Scope, Symbol, SymbolKey, SymbolKind},
    Address,
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

#[cfg_attr(feature = "cli", derive(ValueEnum))]
#[derive(Default, PartialOrd, PartialEq, Ord, Eq, Copy, Clone, Debug)]
pub enum DefSymDataType {
    U8,
    U16,
    #[default]
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
}

impl Display for DefSymDataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefSymDataType::U8 => write!(f, "u8"),
            DefSymDataType::U16 => write!(f, "u16"),
            DefSymDataType::U32 => write!(f, "u32"),
            DefSymDataType::U64 => write!(f, "u64"),
            DefSymDataType::I8 => write!(f, "i8"),
            DefSymDataType::I16 => write!(f, "i16"),
            DefSymDataType::I32 => write!(f, "i32"),
            DefSymDataType::I64 => write!(f, "i64"),
        }
    }
}

#[cfg_attr(feature = "cli", derive(Args))]
#[derive(Clone, Debug, Default)]
pub struct DefSym {
    #[cfg_attr(feature = "cli", clap(long, short))]
    from: Option<Address>,
    #[cfg_attr(feature = "cli", clap(long, short))]
    to: Option<Address>,
    #[cfg_attr(feature = "cli", clap(long, short))]
    const_value: bool,
    name: String,
    value: i64,
    data_type: DefSymDataType,
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
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<SymbolKey> for DefSym {
    fn into(self) -> SymbolKey {
        match self.data_type {
            DefSymDataType::U8 => SymbolKey::U8(self.value as u8),
            DefSymDataType::U16 => SymbolKey::U16(self.value as u16),
            DefSymDataType::U32 => SymbolKey::U32(self.value as u32),
            DefSymDataType::U64 => SymbolKey::U64(self.value as u64),
            DefSymDataType::I8 => SymbolKey::I8(self.value as i8),
            DefSymDataType::I16 => SymbolKey::I16(self.value as i16),
            DefSymDataType::I32 => SymbolKey::I32(self.value as i32),
            DefSymDataType::I64 => SymbolKey::I64(self.value),
        }
    }
}

#[cfg_attr(feature = "cli", derive(Subcommand))]
#[derive(Clone, Debug)]
pub enum Commands {
    Org { address: Address },
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
            ArchKind::ArchCustom => ron::from_str(&std::fs::read_to_string(
                cfg.arch_file.as_ref().expect("No arch file found"),
            )?)
            .expect("Error parsing arch file"),
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

    #[cfg_attr(feature = "cli", arg(long, short))]
    pub start_read: Option<usize>,

    #[cfg_attr(feature = "cli", arg(long, short))]
    pub end_read: Option<usize>,

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
