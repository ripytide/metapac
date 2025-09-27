use std::collections::{BTreeMap, BTreeSet};

use crate::prelude::*;

pub trait BackendEx: Backend {
    /// If possible the backend will attempt to decide whether the given package is a valid package
    /// or not.
    ///
    /// Validity is defined as the package being able to be installed on the current system as the
    /// package manager is currently configured.
    ///
    /// - `Some(true)` means the package is valid
    /// - `Some(false)` means the package is invalid
    /// - `None` means the package could be valid or invalid.
    fn are_packages_valid(
        packages: &BTreeSet<String>,
        config: &Self::Config,
    ) -> BTreeMap<String, Option<bool>>;
}

impl<T> BackendEx for T
where
    T: Backend,
{
    fn are_packages_valid(
        packages: &BTreeSet<String>,
        config: &Self::Config,
    ) -> BTreeMap<String, Option<bool>> {
        let existing_packages: Result<BTreeSet<String>, _> = Self::get_all(config);

        let mut output = BTreeMap::new();
        for package in packages {
            let valid = match &existing_packages {
                Ok(existing_packages) => Some(existing_packages.contains(package)),
                Err(_) => {
                    if Self::is_valid_package_name(package) == Some(false) {
                        Some(false)
                    } else {
                        None
                    }
                }
            };

            output.insert(package.to_string(), valid);
        }

        output
    }
}
