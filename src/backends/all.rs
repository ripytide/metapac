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
            PackageIds {
                $( $backend: self.$backend.keys().cloned().collect() ),*
            }
        }
    };
}

macro_rules! any {
    ($($backend:ident),*) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::FromStr, derive_more::Display)]
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

macro_rules! raw_package_ids {
    ($($backend:ident),*) => {
        #[derive(Debug, Clone, Default)]
        #[allow(non_snake_case)]
        pub struct RawPackageIds {
            $(
                pub $backend: Vec<String>,
            )*
        }
        impl RawPackageIds {
            pub fn contains(&self, backend: AnyBackend, package: &str) -> bool {
                match backend {
                    $( AnyBackend::$backend => self.$backend.iter().any(|p| p == package) ),*
                }
            }
        }
    }
}
apply_public_backends!(raw_package_ids);

macro_rules! package_ids {
    ($($backend:ident),*) => {
        #[derive(Debug, Clone, Default, Serialize)]
        #[allow(non_snake_case)]
        pub struct PackageIds {
            $(
                #[serde(skip_serializing_if = "BTreeSet::is_empty")]
                pub $backend: BTreeSet<String>,
            )*
        }
        impl PackageIds {
            append!($($backend),*);
            is_empty!($($backend),*);

            pub fn contains(&self, backend: AnyBackend, package: &str) -> bool {
                match backend {
                    $( AnyBackend::$backend => self.$backend.contains(package) ),*
                }
            }

            pub fn remove(&mut self, backend: AnyBackend, package: &str) -> bool {
                match backend {
                    $( AnyBackend::$backend => self.$backend.remove(package) ),*
                }
            }

            pub fn difference(&self, other: &Self) -> Self {
                let mut output = Self::default();

                $(
                    output.$backend = self.$backend.difference(&other.$backend).cloned().collect();
                )*

                output
            }

            pub fn remove_packages(&self, no_confirm: bool, config: &Config) -> Result<()> {
                $(
                    if is_enabled(AnyBackend::$backend, config) {
                        AnyBackend::$backend.remove_packages(&self.$backend, no_confirm, config)?;
                    }
                )*

                Ok(())
            }
        }
        impl std::fmt::Display for PackageIds {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $(
                    if !self.$backend.is_empty() {
                        writeln!(f, "[{}]", AnyBackend::$backend)?;
                        for package_id in self.$backend.iter() {
                            writeln!(f, "{package_id}")?;
                        }
                        writeln!(f)?;
                    }
                )*

                Ok(())
            }
        }
    }
}
apply_public_backends!(package_ids);

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
                RawPackageIds {
                    $( $backend: self.$backend.iter().map(|(x, _)| x).cloned().collect() ),*
                }
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
