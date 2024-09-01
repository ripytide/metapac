pub mod apt;
pub mod arch;
pub mod cargo;
pub mod dnf;
pub mod flatpak;
pub mod pip;
pub mod pipx;
pub mod rustup;
pub mod xbps;

use std::{collections::BTreeMap, str::FromStr};

pub use crate::packages::AnyBackend;
use crate::prelude::*;
use anyhow::{Context, Result};
use strum::IntoEnumIterator;

impl FromStr for AnyBackend {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        Self::iter()
            .find(|x| x.to_string().to_lowercase() == s)
            .with_context(|| anyhow::anyhow!("unable to parse backend from string: {s}"))
    }
}

/// A trait to represent any package manager backend
#[enum_dispatch::enum_dispatch]
pub trait Backend {
    type PackageId: TryFrom<String>;
    type InstallOptions;
    type RemoveOptions;
    type QueryInfo;
    type Modification;

    /// Query all packages that are installed in the backend.
    ///
    /// # Errors
    ///
    /// This function shall return an error if the installed packages cannot be
    /// determined.
    fn query_installed_packages(
        config: &Config,
    ) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>>;

    /// Install the specified packages. If `no_confirm` is `true`, pass the corresponding
    /// switch to the package manager. Return the [`ExitStatus`] from the package manager.
    ///
    /// # Errors
    ///
    /// This function will return an error if the package manager cannot be run or it
    /// returns an error.
    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()>;

    /// Modify the packages as specified by [`Self::PackageModification`].
    ///
    /// This may not include installing or removing the package as [`Backend::install_packages()`]
    /// and [`Backend::remove_packages()`] exist for this purpose.
    ///
    /// # Errors
    ///
    /// Returns an error if the backend fails to modify the packages as required.
    fn modify_packages(
        packages: &BTreeMap<Self::PackageId, Self::Modification>,
        config: &Config,
    ) -> Result<()>;

    /// Remove the specified packages.
    ///
    /// # Errors
    ///
    /// Returns an error if the external command fails.
    fn remove_packages(
        packages: &BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()>;
}
