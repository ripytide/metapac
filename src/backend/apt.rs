use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::{command_found, run_args, run_args_for_stdout};
use crate::prelude::*;

#[derive(Debug, Copy, Clone, Default, derive_more::Display)]
pub struct Apt;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AptQueryInfo {
    pub explicit: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AptModification {
    make_implicit: bool,
}

impl Backend for Apt {
    type PackageId = String;
    type RemoveOptions = ();
    type InstallOptions = ();
    type QueryInfo = AptQueryInfo;
    type Modification = AptModification;

    fn query_installed_packages(_: &Config) -> Result<BTreeMap<Self::PackageId, Self::QueryInfo>> {
        if !command_found("apt-mark") {
            return Ok(BTreeMap::new());
        }

        // See https://askubuntu.com/questions/2389/how-to-list-manually-installed-packages
        // for a run-down of methods for finding lists of
        // explicit/dependency packages. It doesn't seem as if apt was
        // designed with this use-case in mind so there are lots and
        // lots of different methods all of which seem to have
        // caveats.
        let explicit = run_args_for_stdout(["apt-mark", "showmanual"])?;
        let dependency = run_args_for_stdout(["apt-mark", "showauto"])?;

        Ok(dependency
            .lines()
            .map(|x| (x.to_string(), AptQueryInfo { explicit: false }))
            .chain(
                explicit
                    .lines()
                    .map(|x| (x.to_string(), AptQueryInfo { explicit: true })),
            )
            .collect())
    }

    fn install_packages(
        packages: &BTreeMap<Self::PackageId, Self::InstallOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["apt-get", "install"]
                .into_iter()
                .chain(Some("--yes").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }

    fn modify_packages(
        packages: &BTreeMap<Self::PackageId, Self::Modification>,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["apt-mark", "auto"].into_iter().chain(
                packages
                    .iter()
                    .filter(|(_, m)| m.make_implicit)
                    .map(|(p, _)| p.as_str()),
            ),
        )
    }

    fn remove_packages(
        packages: &BTreeMap<Self::PackageId, Self::RemoveOptions>,
        no_confirm: bool,
        _: &Config,
    ) -> Result<()> {
        run_args(
            ["apt-get", "remove"]
                .into_iter()
                .chain(Some("--yes").filter(|_| no_confirm))
                .chain(packages.keys().map(String::as_str)),
        )
    }
}
