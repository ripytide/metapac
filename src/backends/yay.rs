use crate::prelude::*;
use anyhow::Result;
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, derive_more::Display)]
pub struct Yay;
impl Yay {
    const YAY: Arch = Arch { command: "yay" };
}

impl Backend for Yay {
    type PackageId = ArchPackageId;
    type QueryInfo = ArchQueryInfo;
    type InstallOptions = ArchInstallOptions;
    type ModificationOptions = ArchModificationOptions;
    type RemoveOptions = ArchRemoveOptions;

    fn query_installed_packages(
        config: &Config,
    ) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        Self::YAY.query_installed_packages(config)
    }
    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        Self::YAY.install_packages(packages, no_confirm, config)
    }
    fn modify_packages(
        packages: &BTreeMap<Self::PackageId, Self::ModificationOptions>,
        config: &Config,
    ) -> Result<()> {
        Self::YAY.modify_packages(packages, config)
    }
    fn remove_packages(
        packages: &BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
        config: &Config,
    ) -> Result<()> {
        Self::YAY.remove_packages(packages, no_confirm, config)
    }
    fn try_parse_toml_package(
            toml: &toml::Value,
        ) -> Result<(Self::PackageId, Self::InstallOptions)> {
        Self::YAY.try_parse_toml_package(toml)
    }
}
