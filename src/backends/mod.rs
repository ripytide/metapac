pub mod all;
pub mod apt;
pub mod arch;
pub mod brew;
pub mod bun;
pub mod cargo;
pub mod dnf;
pub mod flatpak;
pub mod mise;
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
        (Arch, arch),
        (Apt, apt),
        (Brew, brew),
        (Bun, bun),
        (Cargo, cargo),
        (Dnf, dnf),
        (Flatpak, flatpak),
        (Mise, mise),
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

    /// Attempts to expand group packages keeping the same options.
    ///
    /// This uses archlinux's definition for a group package. Note that a group package is distict
    /// from a meta package, see <https://wiki.archlinux.org/title/Meta_package_and_package_group>
    fn expand_group_packages(
        packages: BTreeMap<String, Package<Self::Options>>,
        config: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>>;

    /// Attempts to query which packages are explicitly installed along with their options.
    ///
    /// If a backend cannot distinguish between explicit and implicit packages then it should
    /// return both implicit and explicit packages.
    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>>;

    /// Attempts to explicitly install the given `packages`, optionally without confirmation using
    /// `no_confirm`.
    ///
    /// If any of the `packages` are already installed then this method should return an error without
    /// installing any packages.
    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()>;

    /// Attempts to uninstall the given `packages`, optionally without confirmation using
    /// `no_confirm`.
    ///
    /// If any of the `packages` are not installed then this method should return an error without
    /// uninstalling any packages.
    ///
    /// If the backend supports it this method should also remove any implicit dependencies that
    /// are no longer required by any explicitly installed packages.
    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()>;

    /// Attempts to update the given `packages`, optionally without confirmation using
    /// `no_confirm`.
    ///
    /// If any of the `packages` are not installed then this method should return an error without
    /// updating any packages.
    ///
    /// If the backend supports it this method should try to preserve the existing options that
    /// each package is currently installed with.
    fn update(packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()>;

    /// Attempts to update all packages currently installed, optionally without confirmation using
    /// `no_confirm`.
    ///
    /// If the backend supports it this method should try to preserve the existing options that
    /// each package is currently installed with.
    fn update_all(no_confirm: bool, config: &Config) -> Result<()>;

    /// Attempts to clean all cache.
    fn clean_cache(config: &Config) -> Result<()>;

    /// Attempts to return the version of the backend.
    ///
    /// If the package is not installed then this method should return an error.
    fn version(config: &Config) -> Result<String>;
}
