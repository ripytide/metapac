use color_eyre::Result;
use serde::{Deserialize, Serialize};

use crate::cmd::run_command;
use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hooks {
    pub before_install: Option<Vec<String>>,
    pub after_install: Option<Vec<String>>,
    pub after_sync: Option<Vec<String>>,
    pub before_sync: Option<Vec<String>>,
}
impl Hooks {
    pub fn run_before_install(&self) -> Result<()> {
        if let Some(args) = &self.before_install {
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_after_install(&self) -> Result<()> {
        if let Some(args) = &self.after_install {
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_after_sync(&self) -> Result<()> {
        if let Some(args) = &self.after_sync {
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
    pub fn run_before_sync(&self) -> Result<()> {
        if let Some(args) = &self.before_sync {
            run_command(args, Perms::Same)
        } else {
            Ok(())
        }
    }
}
