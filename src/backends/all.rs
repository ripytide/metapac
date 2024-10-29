use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::prelude::*;
use color_eyre::Result;

macro_rules! append {
    ($($backend:ident),*) => {
        pub fn append(&mut self, other: &mut Self) {
            $(
                self.$backend.append(&mut other.$backend);
            )*
        }
    };
}
macro_rules! is_empty {
    ($($backend:ident),*) => {
        pub fn is_empty(&self) -> bool {
            $(
                self.$backend.is_empty() &&
            )* true
        }
    };
}
macro_rules! to_package_ids {
    ($($backend:ident),*) => {
        pub fn to_package_ids(&self) -> PackageIds {
            PackageIds (
                 BTreeMap::from([
                    $( (AnyBackend::$backend, self.$backend.keys().cloned().collect()), )*
                ])
            )
        }
    };
}

macro_rules! any {
    ($($backend:ident),*) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, derive_more::FromStr, derive_more::Display)]
        pub enum AnyBackend {
            $($backend,)*
        }
        impl AnyBackend {
            pub fn remove_packages(&self, packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()> {
                match self {
                    $( AnyBackend::$backend => $backend::remove_packages(packages, no_confirm, config), )*
                }
            }
        }
    };
}
apply_public_backends!(any);

#[derive(Debug, Clone, Default, Serialize, derive_more::Deref, derive_more::DerefMut)]
#[serde(transparent)]
pub struct RawPackageIds(BTreeMap<AnyBackend, Vec<String>>);
impl RawPackageIds {
    pub fn contains(&self, backend: AnyBackend, package: &String) -> bool {
        self.get(&backend).is_some_and(|x| x.contains(package))
    }
}

#[derive(Debug, Clone, Default, Serialize, derive_more::Deref, derive_more::DerefMut)]
#[serde(transparent)]
pub struct PackageIds(BTreeMap<AnyBackend, BTreeSet<String>>);
impl PackageIds {
    pub fn append(&mut self, other: &mut Self) {
        for (backend, packages) in other.iter_mut() {
            self.entry(*backend).or_default().append(packages);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.values().all(|x| x.is_empty())
    }

    pub fn contains(&self, backend: AnyBackend, package: &str) -> bool {
        self.get(&backend).is_some_and(|x| x.contains(package))
    }

    pub fn remove(&mut self, backend: AnyBackend, package: &str) -> bool {
        if let Some(packages) = self.get_mut(&backend) {
            packages.remove(package)
        } else {
            false
        }
    }

    pub fn difference(&self, other: &Self) -> Self {
        let mut output = Self::default();
        for (backend, packages) in self.iter() {
            if let Some(other_packages) = other.get(backend) {
                output.0.insert(
                    *backend,
                    packages.difference(other_packages).cloned().collect(),
                );
            }
        }
        output
    }

    pub fn remove_packages(&self, no_confirm: bool, config: &Config) -> Result<()> {
        for (backend, packages) in self.0.iter() {
            if is_enabled(*backend, config) {
                backend.remove_packages(packages, no_confirm, config)?;
            }
        }

        Ok(())
    }
}
impl std::fmt::Display for PackageIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (backend, packages) in self.iter() {
            if !packages.is_empty() {
                writeln!(f, "[{backend}]")?;
                for package in packages {
                    writeln!(f, "{package}")?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

macro_rules! query_infos {
    ($($backend:ident),*) => {
        #[derive(Debug, Clone, Default)]
        #[allow(non_snake_case)]
        pub struct QueryInfos {
            $(
                pub $backend: BTreeMap<String, <$backend as Backend>::QueryInfo>,
            )*
        }
        impl QueryInfos {
            append!($($backend),*);
            is_empty!($($backend),*);
            to_package_ids!($($backend),*);

            pub fn query_installed_packages(config: &Config) -> Result<Self> {
                Ok(Self {
                    $(
                        $backend:
                            if is_enabled(AnyBackend::$backend, config) {
                                $backend::query_installed_packages(config)?
                            } else {
                                Default::default()
                            },
                    )*
                })
            }
        }
    }
}
apply_public_backends!(query_infos);

macro_rules! raw_install_options {
    ($($backend:ident),*) => {
        #[derive(Debug, Clone, Default)]
        #[allow(non_snake_case)]
        pub struct RawInstallOptions {
            $(
                pub $backend: Vec<(String, <$backend as Backend>::InstallOptions)>,
            )*
        }
        impl RawInstallOptions {
            append!($($backend),*);

            pub fn to_raw_package_ids(&self) -> RawPackageIds {
                RawPackageIds(
                    BTreeMap::from([
                        $( (AnyBackend::$backend, self.$backend.iter().map(|(x, _)| x).cloned().collect()), )*
                    ])
                )
            }
        }
    }
}
apply_public_backends!(raw_install_options);

macro_rules! install_options {
    ($($backend:ident),*) => {
        #[derive(Debug, Clone, Default)]
        #[allow(non_snake_case)]
        pub struct InstallOptions {
            $(
                pub $backend: BTreeMap<String, <$backend as Backend>::InstallOptions>,
            )*
        }
        impl InstallOptions {
            append!($($backend),*);
            is_empty!($($backend),*);
            to_package_ids!($($backend),*);

            pub fn map_install_packages(mut self, config: &Config) -> Result<Self> {
                $(
                    if is_enabled(AnyBackend::$backend, config) {
                        self.$backend = $backend::map_managed_packages(self.$backend, config)?;
                    }
                )*

                Ok(self)
            }

            pub fn install_packages(self, no_confirm: bool, config: &Config) -> Result<()> {
                $(
                    if is_enabled(AnyBackend::$backend, config) {
                        $backend::install_packages(&self.$backend, no_confirm, config)?;
                    }
                )*

                Ok(())
            }
        }
    }
}
apply_public_backends!(install_options);

fn is_enabled(backend: AnyBackend, config: &Config) -> bool {
    !config
        .disabled_backends
        .iter()
        .any(|x| x.to_lowercase() == backend.to_string().to_lowercase())
}
