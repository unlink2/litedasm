use std::{
    fmt::Display,
    io::{BufReader, BufWriter, LineWriter, Read, Write},
    path::PathBuf,
};

use super::dasm::arch::{a6502, a65c02, a65c816, Archs};
use crate::prelude::FdResult;
#[cfg(feature = "cli")]
use clap::{CommandFactory, Parser, ValueEnum};
#[cfg(feature = "cli")]
use clap_complete::{generate, Generator, Shell};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref CFG: Config = Config::new();
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
    pub fn to_arch(&self, _cfg: &Config) -> FdResult<Archs> {
        Ok(match self {
            ArchKind::Arch6502 => a6502::ARCH.to_owned(),
            ArchKind::Arch65c02 => a65c02::ARCH.to_owned(),
            ArchKind::Arch65c816 => a65c816::ARCH.to_owned(),
            ArchKind::ArchCustom => todo!("Custom archs are not yet supported!"),
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
    pub input: Option<PathBuf>,
    pub output: Option<PathBuf>,

    // built in arch that may be loaded
    #[cfg_attr(feature = "cli", clap(long, short))]
    #[cfg_attr(feature = "cli", arg(default_value_t))]
    pub arch: ArchKind,

    // custom arch config file to load
    #[cfg_attr(feature = "cli", clap(long))]
    pub arch_file: Option<PathBuf>,

    #[cfg_attr(feature = "cli", arg(short, long, action = clap::ArgAction::Count))]
    pub verbose: u8,

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

#[cfg(feature = "cli")]
pub fn generate_completion<G: Generator>(gen: G) {
    generate(
        gen,
        &mut Config::command(),
        Config::command().get_name(),
        &mut std::io::stdout(),
    );
}
