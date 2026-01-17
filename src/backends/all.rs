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

macro_rules! any_backend {
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
apply_backends!(any_backend);

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

macro_rules! all_raw_complex_backend_items {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default, Serialize)]
        pub struct AllRawComplexBackendItems {
            $(
                pub $lower_backend: RawComplexBackendItems<<$upper_backend as Backend>::PackageOptions, <$upper_backend as Backend>::RepoOptions>,
            )*
        }
        impl AllRawComplexBackendItems {
            append!($(($upper_backend, $lower_backend)),*);

            pub fn to_string_pretty(&self) -> Result<String> {
                let mut document = toml_edit::ser::to_document(&self)?;

                $(
                    let array = document.get_mut(&AnyBackend::$upper_backend.to_string().to_lowercase()).unwrap().as_array_mut().unwrap();
                    for (index, item) in self.$lower_backend.packages.iter().enumerate() {
                        let inline_table = array.get_mut(index).unwrap().as_inline_table_mut().unwrap();

                        if item.options == <$upper_backend as Backend>::PackageOptions::default() {
                            inline_table.remove("options");
                        }

                        if item.hooks == Hooks::default() {
                            inline_table.remove("hooks");
                        }

                        if inline_table.len() == 1 {
                            array.replace(index, item.name.to_string());
                        }
                    }
                )*

                document.retain(|_, y| !y.as_array().unwrap().is_empty());

                let unformatted = document.to_string();

                let formatted = taplo::formatter::format(&unformatted, taplo::formatter::Options::default());

                Ok(formatted)
            }
        }
    }
}
apply_backends!(all_raw_complex_backend_items);

macro_rules! all_complex_backend_items {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct AllComplexBackendItems {
            $(
                pub $lower_backend: ComplexBackendItems<<$upper_backend as Backend>::PackageOptions, <$upper_backend as Backend>::RepoOptions>,
            )*
        }
        impl AllComplexBackendItems {
            is_empty!($(($upper_backend, $lower_backend)),*);

            pub fn to_package_ids(self) -> PackageIds {
                PackageIds {
                    $( $lower_backend: self.$lower_backend.to_packages().into_keys().collect() ),*
                }
            }

            pub fn to_raw(self) -> AllRawComplexBackendItems {
                AllRawComplexBackendItems {
                    $(
                        $lower_backend: self.$lower_backend.to_raw(),
                    )*
                }
            }

        }
    }
}
apply_backends!(all_complex_backend_items);

macro_rules! all_backend_items {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct AllBackendItems {
            $(
                pub $lower_backend: BackendItems<<$upper_backend as Backend>::PackageOptions, <$upper_backend as Backend>::RepoOptions>,
            )*
        }
        impl AllBackendItems {
            is_empty!($(($upper_backend, $lower_backend)),*);

            pub fn install_packages(&self, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                $(
                    let packages = BTreeMap::<String, <$upper_backend as Backend>::PackageOptions>::from_iter(self.$lower_backend.packages.iter().map(|(x, y)| (x.to_string(), y.clone())));
                    $upper_backend::install(&packages, no_confirm, &config.$lower_backend)?;
                )*

                Ok(())
            }

            pub fn uninstall_packages(&self, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                $(
                    $upper_backend::uninstall(&self.$lower_backend.packages.keys().cloned().collect(), no_confirm, &config.$lower_backend)?;
                )*

                Ok(())
            }

            pub fn add_repos(&self, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                $(
                    let repos = BTreeMap::<String, <$upper_backend as Backend>::RepoOptions>::from_iter(self.$lower_backend.repos.iter().map(|(x, y)| (x.to_string(), y.clone())));
                    $upper_backend::add_repos(&repos, no_confirm, &config.$lower_backend)?;
                )*

                Ok(())
            }

            pub fn remove_repos(&self, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                $(
                    $upper_backend::remove_repos(&self.$lower_backend.repos.keys().cloned().collect(), no_confirm, &config.$lower_backend)?;
                )*

                Ok(())
            }
        }
    }
}
apply_backends!(all_backend_items);

macro_rules! backend_configs {
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
apply_backends!(backend_configs);
