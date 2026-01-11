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
    #[arg(long)]
    /// specify a different hostname
    pub hostname: Option<String>,
    #[arg(long)]
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
    Update(UpdateCommand),
    UpdateAll(UpdateAllCommand),
    Clean(CleanCommand),
    Sync(SyncCommand),
    Unmanaged(UnmanagedCommand),
    Backends(BackendsCommand),
    CleanCache(CleanCacheCommand),
}

#[derive(Args)]
/// add packages for the given backend and group file
///
/// if the group file does not exist it will be created
pub struct AddCommand {
    #[arg(long)]
    /// the backend for the packages
    pub backend: AnyBackend,
    #[arg(long, required=true, num_args=1..)]
    /// the package names
    pub packages: Vec<String>,
    #[arg(long, default_value = "default")]
    /// the group name
    pub group: String,
}

#[derive(Args)]
/// remove packages for the given backend from all active group files
pub struct RemoveCommand {
    #[arg(long)]
    /// the backend for the packages
    pub backend: AnyBackend,
    #[arg(long, required=true, num_args=1..)]
    /// the package names
    pub packages: Vec<String>,
}

#[derive(Args)]
/// install packages for the given backend and add it to the given group file
///
/// if the group file does not exist it will be created
pub struct InstallCommand {
    #[arg(long)]
    /// the backend for the packages
    pub backend: AnyBackend,
    #[arg(long, required=true, num_args=1..)]
    /// the package names
    pub packages: Vec<String>,
    #[arg(long, default_value = "default")]
    /// the group name
    pub group: String,
    #[arg(long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
/// uninstall packages for the given backend and remove it from all active group files
pub struct UninstallCommand {
    #[arg(long)]
    /// the backend for the packages
    pub backend: AnyBackend,
    #[arg(long, required=true, num_args=1..)]
    /// the package names
    pub packages: Vec<String>,
    #[arg(long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
/// update packages for the given backend
pub struct UpdateCommand {
    #[arg(long)]
    /// the backend for the packages
    pub backend: AnyBackend,
    #[arg(long, required=true, num_args=1..)]
    /// the package names
    pub packages: Vec<String>,
    #[arg(long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
/// update all packages for the given backends
pub struct UpdateAllCommand {
    #[arg(long)]
    /// the backends to operate on
    ///
    /// - if no backends are passed then the enabled_backend config is used
    ///
    /// - if "all" is passed by itself then all backends are used
    ///
    /// - otherwise the list will be parsed as a list of backends to be used
    pub backends: Vec<String>,
    #[arg(long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
/// uninstall unmanaged packages
pub struct CleanCommand {
    #[arg(long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
/// install missing packages from groups
pub struct SyncCommand {
    #[arg(long)]
    /// do not ask for any confirmation
    pub no_confirm: bool,
}

#[derive(Args)]
/// show explicitly installed packages not required by metapac
///
/// the output is in valid toml group file format to allow writing
/// the output to a file which can help in importing packages
/// installed on your system into your group files
pub struct UnmanagedCommand {}

#[derive(Args)]
/// show the backends found by metapac
pub struct BackendsCommand {}

#[derive(Args)]
/// clean the caches for the given backends
pub struct CleanCacheCommand {
    #[arg(long)]
    /// the backends to operate on
    ///
    /// - if no backends are passed then the enabled_backend config is used
    ///
    /// - if "all" is passed by itself then all backends are used
    ///
    /// - otherwise the list will be parsed as a list of backends to be used
    pub backends: Vec<String>,
}
