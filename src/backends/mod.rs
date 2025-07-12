pub mod all;
pub mod apt;
pub mod arch;
pub mod brew;
pub mod cargo;
pub mod dnf;
pub mod flatpak;
pub mod pipx;
pub mod snap;
pub mod uv;
pub mod vscode;
pub mod winget;
pub mod xbps;

use std::collections::{BTreeMap, BTreeSet};

use crate::prelude::*;
use color_eyre::Result;

macro_rules! apply_backends {
    ($macro:ident) => {
        $macro! {
        (Arch, arch),
        (Apt, apt),
        (Brew, brew),
        (Cargo, cargo),
        (Dnf, dnf),
        (Flatpak, flatpak),
        (Pipx, pipx),
        (Snap, snap),
        (Uv, uv),
        (VsCode, vscode),
        (WinGet, winget),
        (Xbps, xbps) }
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
}
