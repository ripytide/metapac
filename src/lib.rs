#![doc = include_str!("../README.md")]

mod backend;
mod cli;

mod cmd;
mod config;
mod core;
mod groups;
mod packages;
mod prelude;
mod review;

pub use prelude::*;
