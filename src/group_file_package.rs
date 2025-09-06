use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::run_command;
use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupFilePackage<T> {
    pub package: String,
    #[serde(default)]
    pub options: T,
    #[serde(default)]
    pub hooks: Hooks,
}
impl<T> GroupFilePackage<T> {
    pub fn into_options(self) -> T {
        self.options
    }

    pub fn run_before_install(&self) -> Result<()> {
        if let Some(args) = &self.hooks.before_install {
            log::info!("running before_install hook for package {:?}", self.package);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_after_install(&self) -> Result<()> {
        if let Some(args) = &self.hooks.after_install {
            log::info!("running after_install hook for package {:?}", self.package);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_after_sync(&self) -> Result<()> {
        if let Some(args) = &self.hooks.after_sync {
            log::info!("running after_sync hook for package {:?}", self.package);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_before_sync(&self) -> Result<()> {
        if let Some(args) = &self.hooks.before_sync {
            log::info!("running before_sync hook for package {:?}", self.package);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hooks {
    pub before_install: Option<Vec<String>>,
    pub after_install: Option<Vec<String>>,
    pub after_sync: Option<Vec<String>>,
    pub before_sync: Option<Vec<String>>,
}
