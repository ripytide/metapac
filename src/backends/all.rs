use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

use crate::prelude::*;
use color_eyre::Result;

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
        #[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, derive_more::FromStr, derive_more::Display, strum::EnumIter)]
        pub enum AnyBackend {
            $($upper_backend,)*
        }
        impl AnyBackend {
	    pub fn clean_cache(&self, config: &Config) -> Result<()> {
		match self {
		    $( AnyBackend::$upper_backend => $upper_backend::clean_cache(config), )*
		}
	    }
            pub fn version(&self, config: &Config) -> Result<String> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::version(config), )*
                }
            }
            pub fn remove(&self, packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::remove(packages, no_confirm, config), )*
                }
            }
        }
    };
}

apply_public_backends!(any);

macro_rules! raw_package_ids {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
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
apply_public_backends!(raw_package_ids);

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

            pub fn difference(&self, other: &Self) -> Self {
                let mut output = Self::default();

                $(
                    output.$lower_backend = self.$lower_backend.difference(&other.$lower_backend).cloned().collect();
                )*

                output
            }

            pub fn remove(&self, no_confirm: bool, config: &Config) -> Result<()> {
                $(
                    if is_enabled(AnyBackend::$upper_backend, config) {
                        AnyBackend::$upper_backend.remove(&self.$lower_backend, no_confirm, config)?;
                    }
                )*

                Ok(())
            }
        }
        impl std::fmt::Display for PackageIds {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $(
                    if !self.$lower_backend.is_empty() {
                        writeln!(f, "[{}]", AnyBackend::$upper_backend)?;
                        for package in self.$lower_backend.iter() {
                            writeln!(f, "{package}")?;
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

macro_rules! raw_options {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct RawOptions {
            $(
                pub $lower_backend: Vec<(String, <$upper_backend as Backend>::Options)>,
            )*
        }
        impl RawOptions {
            append!($(($upper_backend, $lower_backend)),*);

            pub fn to_raw_package_ids(&self) -> RawPackageIds {
                RawPackageIds {
                    $( $lower_backend: self.$lower_backend.iter().map(|(x, _)| x).cloned().collect() ),*
                }
            }
        }
    }
}
apply_public_backends!(raw_options);

macro_rules! options {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
        #[allow(non_snake_case)]
        pub struct Options {
            $(
                pub $lower_backend: BTreeMap<String, <$upper_backend as Backend>::Options>,
            )*
        }
        impl Options {
            append!($(($upper_backend, $lower_backend)),*);
            is_empty!($(($upper_backend, $lower_backend)),*);
            to_package_ids!($(($upper_backend, $lower_backend)),*);

            pub fn map_required(mut self, config: &Config) -> Result<Self> {
                $(
                    if is_enabled(AnyBackend::$upper_backend, config) {
                        self.$lower_backend = $upper_backend::map_required(self.$lower_backend, config)?;
                    }
                )*

                Ok(self)
            }

            pub fn install(self, no_confirm: bool, config: &Config) -> Result<()> {
                $(
                    if is_enabled(AnyBackend::$upper_backend, config) {
                        $upper_backend::install(&self.$lower_backend, no_confirm, config)?;
                    }
                )*

                Ok(())
            }

            pub fn query(config: &Config) -> Result<Self> {
                Ok(Self {
                    $(
                        $lower_backend:
                            if is_enabled(AnyBackend::$upper_backend, config) {
                                $upper_backend::query(config)?
                            } else {
                                Default::default()
                            },
                    )*
                })
            }
        }
    }
}
apply_public_backends!(options);

fn is_enabled(backend: AnyBackend, config: &Config) -> bool {
    !config
        .disabled_backends
        .iter()
        .any(|x| x.to_lowercase() == backend.to_string().to_lowercase())
}
