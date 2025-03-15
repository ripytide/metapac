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
    Add(AddCommand),
    Remove(RemoveCommand),
    Install(InstallCommand),
    Uninstall(UninstallCommand),
    Clean(CleanCommand),
    Sync(SyncCommand),
    Unmanaged(UnmanagedCommand),
    Backends(BackendsCommand),
    CleanCache(CleanCacheCommand),
}

#[derive(Args)]
#[command(visible_alias("c"))]
/// uninstall unmanaged packages
pub struct CleanCommand {
    #[arg(short, long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
#[command(visible_alias("a"))]
/// add packages for the given backend and group file
///
/// if the group file does not exist it will be created
pub struct AddCommand {
    #[arg(short, long)]
    /// the backend for the package
    pub backend: AnyBackend,
    #[arg(short, long)]
    /// the package names
    pub packages: Vec<String>,
    #[arg(short, long, default_value = "default")]
    /// the group name
    pub group: String,
}

#[derive(Args)]
#[command(visible_alias("r"))]
/// remove packages for the given backend from all active group files
pub struct RemoveCommand {
    #[arg(short, long)]
    /// the backend for the package
    pub backend: AnyBackend,
    #[arg(short, long)]
    /// the package names
    pub packages: Vec<String>,
}

#[derive(Args)]
#[command(visible_alias("i"))]
/// install packages for the given backend and add it to the given group file
///
/// if the group file does not exist it will be created
pub struct InstallCommand {
    #[arg(short, long)]
    /// the backend for the package
    pub backend: AnyBackend,
    #[arg(short, long)]
    /// the package names
    pub packages: Vec<String>,
    #[arg(short, long, default_value = "default")]
    /// the group name
    pub group: String,
    #[arg(short, long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
#[command(visible_alias("n"))]
/// uninstall packages for the given backend and remove it from all active group files
pub struct UninstallCommand {
    #[arg(short, long)]
    /// the backend for the package
    pub backend: AnyBackend,
    #[arg(short, long)]
    /// the package names
    pub packages: Vec<String>,
    #[arg(short, long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

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
/// show explicitly installed packages not required by metapac
///
/// the output is in valid toml group file format to allow writing
/// the output to a file which can help in importing packages
/// installed on your system into your group files
pub struct UnmanagedCommand {}

#[derive(Args)]
#[command(visible_alias("b"))]
/// show the backends found by metapac
pub struct BackendsCommand {}

#[derive(Args)]
#[command(visible_alias("e"))]
/// clean the caches of all the backends, or the just those specified
pub struct CleanCacheCommand {
    pub backends: Option<Vec<String>>,
}
