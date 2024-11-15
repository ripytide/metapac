//! The clap declarative command line interface

use crate::prelude::*;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    version,
    author,
    about,
    arg_required_else_help(true),
    subcommand_required(true)
)]
pub struct MainArguments {
    #[arg(short = 'n', long)]
    /// specify a different hostname
    pub hostname: Option<String>,
    #[arg(short, long)]
    /// specify a different config directory
    pub config_dir: Option<PathBuf>,
    #[command(subcommand)]
    pub subcommand: MainSubcommand,
}

#[derive(Subcommand)]
pub enum MainSubcommand {
    Clean(CleanCommand),
    Add(AddCommand),
    Review(ReviewCommand),
    Sync(SyncCommand),
    Unmanaged(UnmanagedCommand),
    Backends(BackendsCommand),
}

#[derive(Args)]
#[command(visible_alias("c"))]
/// remove unmanaged packages
pub struct CleanCommand {
    #[arg(short, long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
#[command(visible_alias("a"))]
/// add a package for the given backend and group file
///
/// if the group file does not exist a new one will be created
pub struct AddCommand {
    #[arg(short, long)]
    /// the backend for the package
    pub backend: AnyBackend,
    #[arg(short, long)]
    /// the package name
    pub package: String,
    #[arg(short, long, default_value = "default")]
    /// the group name
    pub group: String,
}

#[derive(Args)]
#[command(visible_alias("r"))]
/// review unmanaged packages
pub struct ReviewCommand {}

#[derive(Args)]
#[command(visible_alias("s"))]
/// install packages from groups
pub struct SyncCommand {
    #[arg(short, long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
#[command(visible_alias("u"))]
/// show explicitly installed packages not managed by metapac
pub struct UnmanagedCommand {}

#[derive(Args)]
#[command(visible_alias("b"))]
/// show the backends found by metapac
pub struct BackendsCommand {}
