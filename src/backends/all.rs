use crate::prelude::*;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

macro_rules! append {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        pub fn append(&mut self, other: &mut Self) {
            $(
                self.$lower_backend.append(&mut other.$lower_backend);
            )*
        }
    };
}
macro_rules! is_empty {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        pub fn is_empty(&self) -> bool {
            $(
                self.$lower_backend.is_empty() &&
            )* true
        }
    };
}

macro_rules! to_package_ids {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        pub fn to_package_ids(&self) -> PackageIds {
            PackageIds {
                $( $lower_backend: self.$lower_backend.keys().cloned().collect() ),*
            }
        }
    };
}

macro_rules! any {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::FromStr, derive_more::Display, strum::EnumIter, Serialize, Deserialize)]
        #[serde(rename_all = "lowercase")]
        pub enum AnyBackend {
            $($upper_backend,)*
        }
        impl AnyBackend {
            pub fn clean_cache(&self, config: &BackendConfigs) -> Result<()> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::clean_cache(&config.$lower_backend), )*
                }
            }
            pub fn update(&self, packages: &BTreeSet<String>, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::update(packages, no_confirm, &config.$lower_backend), )*
                }
            }
            pub fn update_all(&self, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::update_all(no_confirm, &config.$lower_backend), )*
                }
            }
            pub fn version(&self, config: &BackendConfigs) -> Result<String> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::version(&config.$lower_backend), )*
                }
            }
        }
    };
}
apply_backends!(any);

macro_rules! raw_package_ids {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default, Serialize)]
        pub struct RawPackageIds {
            $(
                pub $lower_backend: Vec<String>,
            )*
        }
        impl RawPackageIds {
            pub fn contains(&self, backend: AnyBackend, package: &str) -> bool {
                match backend {
                    $( AnyBackend::$upper_backend => self.$lower_backend.iter().any(|p| p == package) ),*
                }
            }
        }
    }
}
apply_backends!(raw_package_ids);

macro_rules! package_ids {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default, Serialize)]
        pub struct PackageIds {
            $(
                #[serde(skip_serializing_if = "BTreeSet::is_empty")]
                pub $lower_backend: BTreeSet<String>,
            )*
        }
        impl PackageIds {
            append!($(($upper_backend, $lower_backend)),*);
            is_empty!($(($upper_backend, $lower_backend)),*);

            pub fn contains(&self, backend: AnyBackend, package: &str) -> bool {
                match backend {
                    $( AnyBackend::$upper_backend => self.$lower_backend.contains(package) ),*
                }
            }
        }
    }
}
apply_backends!(package_ids);

macro_rules! raw_packages {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default, Serialize)]
        pub struct RawGroupFilePackages {
            $(
                pub $lower_backend: Vec<GroupFilePackage<<$upper_backend as Backend>::Options>>,
            )*
        }
        impl RawGroupFilePackages {
            append!($(($upper_backend, $lower_backend)),*);

            pub fn to_raw_package_ids(&self) -> RawPackageIds {
                RawPackageIds {
                    $(
                        $lower_backend: self.$lower_backend.iter().map(|x| x.package.clone()).collect(),
                    )*
                }
            }

            pub fn to_string_pretty(&self) -> Result<String> {
                let mut document = toml_edit::ser::to_document(self)?;
                
                document.retain(|_, y| !y.as_array().unwrap().is_empty());

                Ok(document.to_string())
            }
        }
    }
}
apply_backends!(raw_packages);

macro_rules! group_file_packages {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct GroupFilePackages {
            $(
                pub $lower_backend: BTreeMap<String, GroupFilePackage<<$upper_backend as Backend>::Options>>,
            )*
        }
        impl GroupFilePackages {
            append!($(($upper_backend, $lower_backend)),*);
            is_empty!($(($upper_backend, $lower_backend)),*);
            to_package_ids!($(($upper_backend, $lower_backend)),*);

            pub fn to_raw(&self) -> RawGroupFilePackages {
                RawGroupFilePackages {
                    $(
                        $lower_backend: self.$lower_backend.values().cloned().collect(),
                    )*
                }
            }

            pub fn to_packages(&self) -> Packages {
                Packages {
                    $(
                        $lower_backend: self.$lower_backend.iter().map(|(x, y)| (x.to_string(), y.options.clone())).collect(),
                    )*
                }
            }
        }
    }
}
apply_backends!(group_file_packages);

macro_rules! packages {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct Packages {
            $(
                pub $lower_backend: BTreeMap<String, <$upper_backend as Backend>::Options>,
            )*
        }
        impl Packages {
            is_empty!($(($upper_backend, $lower_backend)),*);

            pub fn install(&self, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                $(
                    let options = BTreeMap::<String, <$upper_backend as Backend>::Options>::from_iter(self.$lower_backend.iter().map(|(x, y)| (x.to_string(), y.clone())));
                    $upper_backend::install(&options, no_confirm, &config.$lower_backend)?;
                )*

                Ok(())
            }

            pub fn uninstall(&self, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                $(
                    $upper_backend::uninstall(&self.$lower_backend.keys().cloned().collect(), no_confirm, &config.$lower_backend)?;
                )*

                Ok(())
            }

            pub fn to_group_file_packages(&self) -> GroupFilePackages {
                GroupFilePackages {
                    $(
                        $lower_backend: self.$lower_backend.iter().map(|(x, y)| (x.to_string(), GroupFilePackage {package: x.to_string(), options: y.clone(), hooks: Hooks::default()})).collect(),
                    )*
                }
            }
        }
    }
}
apply_backends!(packages);

macro_rules! configs {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Serialize, Deserialize, Default)]
        #[serde(deny_unknown_fields)]
        pub struct BackendConfigs {
            $(
                #[serde(default)]
                pub $lower_backend: <$upper_backend as Backend>::Config,
            )*
        }
    }
}
apply_backends!(configs);
