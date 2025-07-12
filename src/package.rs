use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::run_command;
use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package<T> {
    pub package: String,
    pub options: Option<T>,
    pub hooks: Option<Hooks>,
}
impl<T> Package<T> {
    pub fn into_options(self) -> Option<T> {
        self.options
    }

    pub fn run_before_install(&self) -> Result<()> {
        if let Some(hooks) = &self.hooks
            && let Some(args) = &hooks.before_install
        {
            log::info!("running before_install hook for {} package", self.package);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_after_install(&self) -> Result<()> {
        if let Some(hooks) = &self.hooks
            && let Some(args) = &hooks.after_install
        {
            log::info!("running after_install hook for {} package", self.package);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_before_uninstall(&self) -> Result<()> {
        if let Some(hooks) = &self.hooks
            && let Some(args) = &hooks.before_uninstall
        {
            log::info!("running before_uninstall hook for {} package", self.package);
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_after_uninstall(&self) -> Result<()> {
        if let Some(hooks) = &self.hooks
            && let Some(args) = &hooks.after_uninstall 
        {
            log::info!("running after_uninstall hook for {} package", self.package);
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
    pub before_uninstall: Option<Vec<String>>,
    pub after_uninstall: Option<Vec<String>>,
}
