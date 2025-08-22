use crate::prelude::*;
use color_eyre::Result;
use color_eyre::eyre::{Context, eyre};
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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
            pub fn clean_cache(&self, config: &Config) -> Result<()> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::clean_cache(config.as_ref()), )*
                }
            }
            pub fn update(&self, packages: &BTreeSet<String>, no_confirm: bool, config: &Config) -> Result<()> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::update(packages, no_confirm, config.as_ref()), )*
                }
            }
            pub fn update_all(&self, no_confirm: bool, config: &Config) -> Result<()> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::update_all(no_confirm, config.as_ref()), )*
                }
            }
            pub fn version(&self, config: &Config) -> Result<String> {
                match self {
                    $( AnyBackend::$upper_backend => $upper_backend::version(config.as_ref()), )*
                }
            }
        }
    };
}
apply_backends!(any);

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

            pub fn filtered(&self, config: &Config) -> PackageIds {
                let mut packages = self.clone();
                $(
                    if !config.enabled_backends.contains(&AnyBackend::$upper_backend) {
                        packages.$lower_backend = Default::default();
                    }
                )*
                packages
            }

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
apply_backends!(package_ids);

macro_rules! configs {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[serde_inline_default]
        #[derive(Debug, Serialize, Deserialize, Default)]
        #[serde(deny_unknown_fields)]
        pub struct Config {// Update README if fields change.
            #[serde_inline_default(Config::default().enabled_backends)]
            pub enabled_backends: BTreeSet<AnyBackend>,
            #[serde_inline_default(Config::default().hostname_groups_enabled)]
            pub hostname_groups_enabled: bool,
            #[serde_inline_default(Config::default().hostname_groups)]
            pub hostname_groups: BTreeMap<String, Vec<String>>,
            $(
                #[serde_inline_default(Config::default().$lower_backend)]
                pub $lower_backend: <$upper_backend as Backend>::Config,
            )*
        }


impl Config {
    pub fn load(config_dir: &Path) -> Result<Self> {
        let config_file_path = config_dir.join("config.toml");

        if !config_file_path.is_file() {
            log::warn!(
                "no config file found at {config_file_path:?}, using default config instead"
            );

            Ok(Self::default())
        } else {
            toml::from_str(
                &std::fs::read_to_string(config_file_path.clone())
                    .wrap_err("reading config file")?,
            )
            .wrap_err(eyre!("parsing toml config {config_file_path:?}"))
        }
    }
}



        $(
        impl AsRef<<$upper_backend as Backend>::Config> for Config {
            fn as_ref(&self) -> &<$upper_backend as Backend>::Config{
                &self.$lower_backend
            }
        }
        )*
    }
}
apply_backends!(configs);

macro_rules! raw_packages {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct RawPackages {
            $(
                pub $lower_backend: Vec<Package<<$upper_backend as Backend>::Options>>,
            )*
        }
        impl RawPackages {
            append!($(($upper_backend, $lower_backend)),*);

            pub fn to_raw_package_ids(&self) -> RawPackageIds {
                RawPackageIds {
                    $( $lower_backend: self.$lower_backend.iter().map(|x| x.package.clone()).collect() ),*
                }
            }
        }
    }
}
apply_backends!(raw_packages);

macro_rules! packages {
    ($(($upper_backend:ident, $lower_backend:ident)),*) => {
        #[derive(Debug, Clone, Default)]
        pub struct Packages {
            $(
                pub $lower_backend: BTreeMap<String, Package<<$upper_backend as Backend>::Options>>,
            )*
        }
        impl Packages {
            append!($(($upper_backend, $lower_backend)),*);
            is_empty!($(($upper_backend, $lower_backend)),*);
            to_package_ids!($(($upper_backend, $lower_backend)),*);

            pub fn filtered(&self, config: &Config) -> Packages {
                let mut packages = self.clone();
                $(
                    if !config.enabled_backends.contains(&AnyBackend::$upper_backend) {
                        packages.$lower_backend = Default::default();
                    }
                )*
                packages
            }

            pub fn expand_group_packages(mut self, config: &Config) -> Result<Self> {
                $(
                    self.$lower_backend = $upper_backend::expand_group_packages(self.$lower_backend, config.as_ref())?;
                )*

                Ok(self)
            }

            pub fn install(&self, no_confirm: bool, config: &Config) -> Result<()> {
                $(
                    for package in self.$lower_backend.values() {
                        package.run_before_install()?;
                    }

                    let options = BTreeMap::<String, <$upper_backend as Backend>::Options>::from_iter(self.$lower_backend.iter().map(|(x, y)| (x.to_string(), y.clone().into_options().unwrap_or_default())));
                    $upper_backend::install(&options, no_confirm, config.as_ref())?;

                    for package in self.$lower_backend.values() {
                        package.run_after_install()?;
                    }
                )*

                Ok(())
            }

            pub fn uninstall(&self, no_confirm: bool, config: &Config) -> Result<()> {
                $(
                    $upper_backend::uninstall(&self.$lower_backend.keys().cloned().collect(), no_confirm, config.as_ref())?;
                )*

                Ok(())
            }
        }
    }
}
apply_backends!(packages);
