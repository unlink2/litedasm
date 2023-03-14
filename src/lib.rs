#![feature(trait_alias)]
#![feature(let_chains)]
#![feature(str_split_whitespace_remainder)]

#[cfg(feature = "cli")]
pub mod cli;
pub mod core;
pub mod prelude;

#[cfg(feature = "tui")]
pub mod tui;
