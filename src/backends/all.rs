use crate::prelude::*;
use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
                    $( AnyBackend::$upper_backend => $upper_backend::update_packages(packages, no_confirm, &config.$lower_backend), )*
                }
            }
            pub fn update_all(&self, no_confirm: bool, config: &BackendConfigs) -> Result<()> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::update_all_packages(no_confirm, &config.$lower_backend), )*
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
                    let backend_table = document.get_mut(&AnyBackend::$upper_backend.to_string().to_lowercase()).unwrap().as_inline_table_mut().unwrap();

                    let package_array = backend_table.get_mut("packages").unwrap().as_array_mut().unwrap();
                    for (index, item) in self.$lower_backend.packages.iter().enumerate() {
                        let inline_table = package_array.get_mut(index).unwrap().as_inline_table_mut().unwrap();

                        if item.options == <$upper_backend as Backend>::PackageOptions::default() {
                            inline_table.remove("options");
                        }

                        if item.hooks == Hooks::default() {
                            inline_table.remove("hooks");
                        }

                        if inline_table.len() == 1 {
                            package_array.replace(index, item.name.to_string());
                        }
                    }

                    let repos_array = backend_table.get_mut("repos").unwrap().as_array_mut().unwrap();
                    for (index, item) in self.$lower_backend.repos.iter().enumerate() {
                        let inline_table = repos_array.get_mut(index).unwrap().as_inline_table_mut().unwrap();

                        if item.options == <$upper_backend as Backend>::RepoOptions::default() {
                            inline_table.remove("options");
                        }

                        if item.hooks == Hooks::default() {
                            inline_table.remove("hooks");
                        }

                        if inline_table.len() == 1 {
                            repos_array.replace(index, item.name.to_string());
                        }
                    }

                    backend_table.retain(|_, array| !array.as_array().unwrap().is_empty());

                    // reverse the order of packages and repos since I like to have repos before
                    // packages
                    backend_table.sort_values_by(|key1, _, key2, _| key1.cmp(key2).reverse());
                )*

                document.retain(|_, backend| !backend.as_inline_table_mut().unwrap().is_empty());

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

            pub fn to_raw(self) -> AllRawComplexBackendItems {
                AllRawComplexBackendItems {
                    $(
                        $lower_backend: self.$lower_backend.to_raw(),
                    )*
                }
            }

            pub fn to_non_complex(self) -> AllBackendItems {
                AllBackendItems {
                    $(
                        $lower_backend: self.$lower_backend.to_non_complex(),
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

            pub fn to_complex(self) -> AllComplexBackendItems {
                AllComplexBackendItems {
                    $(
                        $lower_backend: self.$lower_backend.to_complex(),
                    )*
                }
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
