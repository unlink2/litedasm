#![feature(trait_alias)]
#![feature(let_chains)]

#[cfg(feature = "cli")]
pub mod cli;
pub mod core;
pub mod prelude;

#[cfg(feature = "tui")]
pub mod tui;
