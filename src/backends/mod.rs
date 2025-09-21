pub mod all;
pub mod apt;
pub mod arch;
pub mod brew;
pub mod bun;
pub mod cargo;
pub mod dnf;
pub mod flatpak;
pub mod npm;
pub mod pipx;
pub mod pnpm;
pub mod scoop;
pub mod snap;
pub mod uv;
pub mod vscode;
pub mod winget;
pub mod xbps;
pub mod yarn;

use std::collections::{BTreeMap, BTreeSet};

use crate::prelude::*;
use color_eyre::Result;

macro_rules! apply_backends {
    ($macro:ident) => {
        $macro! {
        (Apt, apt),
        (Arch, arch),
        (Brew, brew),
        (Bun, bun),
        (Cargo, cargo),
        (Dnf, dnf),
        (Flatpak, flatpak),
        (Npm, npm),
        (Pipx, pipx),
        (Pnpm, pnpm),
        (Scoop, scoop),
        (Snap, snap),
        (Uv, uv),
        (VsCode, vscode),
        (WinGet, winget),
        (Xbps, xbps),
        (Yarn, yarn) }
    };
}
pub(crate) use apply_backends;

pub trait Backend {
    type Options;
    type Config;

    fn invalid_package_help_text() -> String;

    /// If possible the backend will attempt to decide whether the given package is a valid package
    /// or not.
    ///
    /// Validity is defined as the package being able to be installed on the current system as the
    /// package manager is currently configured.
    ///
    /// - `Some(true)` means the package is valid
    /// - `Some(false)` means the package is invalid
    /// - `None` means the package could be valid or invalid.
    fn are_valid_packages(
        packages: &BTreeSet<String>,
        config: &Config,
    ) -> BTreeMap<String, Option<bool>>;

    /// If possible the backend will attempt to decide whether the given package name is valid.
    ///
    /// Validity is defined as agreeing to the documented rules for that backend, such as only
    /// being made up of valid characters. And importantly, another rule specific to metapac is
    /// that if there are two forms of name for the same package (such as `metapac` vs
    /// `main/metapac`) then the implicit package names are always invalid as otherwise it would
    /// cause ambiguity in matching installed packages against a users group files.
    ///
    /// - `Some(true)` means the package name is valid
    /// - `Some(false)` means the package name is invalid
    /// - `None` means the package name could be valid or invalid.
    fn is_valid_package_name(package: &str) -> Option<bool>;

    /// Attempts to return which packages are explicitly installed along with their options.
    ///
    /// If a backend cannot distinguish between explicit and implicit packages then it should
    /// return both implicit and explicit packages.
    fn get_installed(config: &Self::Config) -> Result<BTreeMap<String, Self::Options>>;

    /// Attempts to explicitly install the given `packages`, optionally without confirmation using
    /// `no_confirm`.
    ///
    /// If any of the `packages` are already installed then this method should return an error without
    /// installing any packages.
    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        config: &Self::Config,
    ) -> Result<()>;

    /// Attempts to uninstall the given `packages`, optionally without confirmation using
    /// `no_confirm`.
    ///
    /// If any of the `packages` are not installed then this method should return an error without
    /// uninstalling any packages.
    ///
    /// If the backend supports it this method should also remove any implicit dependencies that
    /// are no longer required by any explicitly installed packages.
    fn uninstall(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        config: &Self::Config,
    ) -> Result<()>;

    /// Attempts to update the given `packages`, optionally without confirmation using
    /// `no_confirm`.
    ///
    /// If any of the `packages` are not installed then this method should return an error without
    /// updating any packages.
    ///
    /// If the backend supports it this method should try to preserve the existing options that
    /// each package is currently installed with.
    fn update(packages: &BTreeSet<String>, no_confirm: bool, config: &Self::Config) -> Result<()>;

    /// Attempts to update all packages currently installed, optionally without confirmation using
    /// `no_confirm`.
    ///
    /// If the backend supports it this method should try to preserve the existing options that
    /// each package is currently installed with.
    fn update_all(no_confirm: bool, config: &Self::Config) -> Result<()>;

    /// Attempts to clean all cache.
    fn clean_cache(config: &Self::Config) -> Result<()>;

    /// Attempts to return the version of the backend.
    ///
    /// If the package is not installed then this method should return an error.
    fn version(config: &Self::Config) -> Result<String>;
}
