pub mod all;
pub mod apt;
pub mod arch;
pub mod brew;
pub mod cargo;
pub mod dnf;
pub mod flatpak;
pub mod pipx;
pub mod rustup;
pub mod snap;
pub mod winget;
pub mod xbps;

use std::collections::{BTreeMap, BTreeSet};

use crate::prelude::*;
use color_eyre::Result;
use serde::{Deserialize, Serialize};

macro_rules! apply_public_backends {
    ($macro:ident) => {
        $macro! {
        (Arch, arch),
        (Apt, apt),
        (Brew, brew),
        (Cargo, cargo),
        (Dnf, dnf),
        (Flatpak, flatpak),
        (Pipx, pipx),
        (Rustup, rustup),
        (Snap, snap),
        (WinGet, winget),
        (Xbps, xbps) }
    };
}
pub(crate) use apply_public_backends;

#[derive(Debug, Serialize, Deserialize)]
pub struct StringPackageStruct {
    pub package: String,
}

pub trait Backend {
    type Options;

    fn map_managed_packages(
        packages: BTreeMap<String, Self::Options>,
        config: &Config,
    ) -> Result<BTreeMap<String, Self::Options>>;

    fn query(config: &Config) -> Result<BTreeMap<String, Self::Options>>;

    fn install(
        packages: &BTreeMap<String, Self::Options>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()>;

    fn remove(
        packages: &BTreeSet<String>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()>;

    fn clean_cache(config: &Config) -> Result<()>;

    fn version(config: &Config) -> Result<String>;

    fn missing(managed: Self::Options, installed: Option<Self::Options>) -> Option<Self::Options>;
}
