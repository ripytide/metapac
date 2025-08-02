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
        (Npm, npm),
        (Pipx, pipx),
        (Pnpm, pnpm),
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

    fn map_required(
        packages: BTreeMap<String, Package<Self::Options>>,
        config: &Config,
    ) -> Result<BTreeMap<String, Package<Self::Options>>>;

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>>;

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()>;

    fn uninstall(packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()>;

    fn clean_cache(config: &Config) -> Result<()>;

    fn version(config: &Config) -> Result<String>;

    fn update(config: &Config) -> Result<()> {
        color_eyre::eyre::bail!("Updating is not yet implemented for this backend")
    }
}
