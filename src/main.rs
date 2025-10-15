//! Main program for `metapac`.

#![warn(
    clippy::as_conversions,
    clippy::option_if_let_else,
    clippy::redundant_pub_crate,
    clippy::semicolon_if_nothing_returned,
    clippy::unnecessary_wraps,
    clippy::unused_self,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::use_self,
    clippy::wildcard_dependencies,
    missing_docs
)]

use clap::Parser;
use color_eyre::Result;
use metapac::MainArguments;

fn main() -> Result<()> {
    pretty_env_logger::init();
    color_eyre::install()?;

    MainArguments::parse().run()
}
