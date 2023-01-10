use std::collections::HashSet;

use anyhow::{bail, Context, Result};
use clap::ArgMatches;

use crate::action;
use crate::backend::{Backend, Backends};
use crate::cmd::run_edit_command;
use crate::ui::get_user_confirmation;
use crate::Group;
use crate::Package;

pub struct Pacdef {
    pub(crate) args: ArgMatches,
    pub(crate) groups: HashSet<Group>,
}

impl Pacdef {
    pub fn new(args: ArgMatches, groups: HashSet<Group>) -> Self {
        Self { args, groups }
    }

    pub(crate) fn install_packages(&self) {
        let mut to_install = ToDoPerBackend::new();

        for mut b in Backends::iter() {
            print!("{}: ", b.get_binary());

            b.load(&self.groups);

            let diff = b.get_missing_packages_sorted();
            if diff.is_empty() {
                println!("nothing to do");
                continue;
            }

            println!("would install the following packages");
            for p in &diff {
                println!("  {p}");
            }
            to_install.push((b, diff));
            println!();
        }

        if to_install.nothing_to_do_for_all_backends() {
            return;
        }

        if !get_user_confirmation() {
            return;
        };

        to_install.install_missing_packages()
    }

    #[allow(clippy::unit_arg)]
    pub fn run_action_from_arg(self) -> Result<()> {
        match self.args.subcommand() {
            // Some((action::CLEAN, _)) => Ok(self.clean_packages()),
            Some((action::EDIT, groups)) => self.edit_group_files(groups).context("editing"),
            Some((action::GROUPS, _)) => Ok(self.show_groups()),
            Some((action::SYNC, _)) => Ok(self.install_packages()),
            Some((action::UNMANAGED, _)) => Ok(self.show_unmanaged_packages()),
            Some((action::VERSION, _)) => Ok(self.show_version()),
            _ => todo!(),
        }
    }

    pub(crate) fn edit_group_files(&self, groups: &ArgMatches) -> Result<()> {
        let files: Vec<_> = groups
            .get_many::<String>("group")
            .context("getting group from args")?
            .map(|file| {
                let mut buf = crate::path::get_pacdef_group_dir().unwrap();
                buf.push(file);
                buf
            })
            .collect();
        if run_edit_command(&files)
            .context("running editor")?
            .success()
        {
            Ok(())
        } else {
            bail!("editor exited with error")
        }
    }

    pub(crate) fn show_version(self) {
        println!("pacdef, version: {}", env!("CARGO_PKG_VERSION"))
    }

    fn show_unmanaged_packages(self) {
        let unmanaged_per_backend = &self.get_unmanaged_packages();

        for (backend, packages) in unmanaged_per_backend.iter() {
            if packages.is_empty() {
                continue;
            }
            println!("{}", backend.get_section());
            for package in packages {
                println!("  {package}");
            }
        }
    }

    fn get_unmanaged_packages(self) -> ToDoPerBackend {
        let mut result = ToDoPerBackend::new();

        for mut backend in Backends::iter() {
            backend.load(&self.groups);

            let unmanaged = backend.get_unmanaged_packages_sorted();
            result.push((backend, unmanaged));
        }
        result
    }

    // /// Returns a `Vec` of alphabetically sorted unmanaged packages.
    // pub(crate) fn get_unmanaged_packages(&mut self) -> Vec<Package> {
    //     let managed = self.take_packages_as_set();
    //     let explicitly_installed = Pacman::get_explicitly_installed_packages();
    //     let mut result: Vec<_> = explicitly_installed
    //         .into_iter()
    //         .filter(|p| !managed.contains(p))
    //         .collect();
    //     result.sort_unstable();
    //     result
    // }

    fn show_groups(self) {
        let mut vec: Vec<_> = self.groups.iter().collect();
        vec.sort_unstable();
        for g in vec {
            println!("{}", g.name);
        }
    }

    // fn clean_packages(mut self) {
    //     let unmanaged = self.get_unmanaged_packages();
    //     if unmanaged.is_empty() {
    //         println!("nothing to do");
    //         return;
    //     }

    //     println!("Would remove the following packages and their dependencies:");
    //     for p in &unmanaged {
    //         println!("  {p}");
    //     }
    //     get_user_confirmation();
    //     Pacman::remove_packages(unmanaged);
    // }
}

struct ToDoPerBackend(Vec<(Box<dyn Backend>, Vec<Package>)>);

impl ToDoPerBackend {
    fn new() -> Self {
        Self(vec![])
    }

    fn push(&mut self, item: (Box<dyn Backend>, Vec<Package>)) {
        self.0.push(item);
    }

    fn iter(&self) -> impl Iterator<Item = &(Box<dyn Backend>, Vec<Package>)> {
        self.0.iter()
    }

    fn nothing_to_do_for_all_backends(&self) -> bool {
        self.0.iter().all(|(_, diff)| diff.is_empty())
    }

    fn install_missing_packages(&self) {
        self.0
            .iter()
            .for_each(|(backend, diff)| backend.install_packages(diff));
    }
}
