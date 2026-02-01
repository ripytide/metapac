/// This contains the four variants of types used with group files
///
/// Raw types use `Vec`s instead of `BTreeMap` since they could contain duplicates.
/// Once duplicate detection is done the type is converted into the non-raw variants.
///
/// Then there is complex vs non-complex where complex items include extra information which at the
/// time of writing is just hooks, but hooks are not information backends should care about so we
/// have variants with and without the extra information which is complex and non-complex
/// respectively.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawComplexBackendItems<P, R> {
    #[serde(default)]
    pub packages: Vec<ComplexItem<P>>,
    #[serde(default)]
    pub repos: Vec<ComplexItem<R>>,
}
impl<P, R> RawComplexBackendItems<P, R> {
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.repos.is_empty()
    }
    pub fn append(&mut self, other: &mut Self) {
        self.packages.append(&mut other.packages);
        self.repos.append(&mut other.repos);
    }
    pub fn to_non_raw(self) -> ComplexBackendItems<P, R> {
        ComplexBackendItems {
            packages: self
                .packages
                .into_iter()
                .map(|x| (x.name.clone(), x))
                .collect(),
            repos: self
                .repos
                .into_iter()
                .map(|x| (x.name.clone(), x))
                .collect(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexBackendItems<P, R> {
    #[serde(default)]
    pub packages: BTreeMap<String, ComplexItem<P>>,
    #[serde(default)]
    pub repos: BTreeMap<String, ComplexItem<R>>,
}
impl<P, R> ComplexBackendItems<P, R> {
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.repos.is_empty()
    }
    pub fn to_non_complex(self) -> BackendItems<P, R> {
        BackendItems {
            packages: self
                .packages
                .into_iter()
                .map(|(x, y)| (x, y.options))
                .collect(),
            repos: self
                .repos
                .into_iter()
                .map(|(x, y)| (x, y.options))
                .collect(),
        }
    }
    pub fn to_raw(self) -> RawComplexBackendItems<P, R> {
        RawComplexBackendItems {
            packages: self.packages.into_values().collect(),
            repos: self.repos.into_values().collect(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendItems<P, R> {
    #[serde(default)]
    pub packages: BTreeMap<String, P>,
    #[serde(default)]
    pub repos: BTreeMap<String, R>,
}
impl<P, R> BackendItems<P, R> {
    pub fn is_empty(&self) -> bool {
        self.packages.is_empty() && self.repos.is_empty()
    }

    pub fn to_complex(self) -> ComplexBackendItems<P, R> {
        ComplexBackendItems {
            packages: self
                .packages
                .into_iter()
                .map(|(x, y)| {
                    (
                        x.clone(),
                        ComplexItem {
                            name: x,
                            options: y,
                            hooks: Hooks::default(),
                        },
                    )
                })
                .collect(),
            repos: self
                .repos
                .into_iter()
                .map(|(x, y)| {
                    (
                        x.clone(),
                        ComplexItem {
                            name: x,
                            options: y,
                            hooks: Hooks::default(),
                        },
                    )
                })
                .collect(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexItem<T> {
    pub name: String,
    #[serde(default)]
    pub options: T,
    #[serde(default)]
    pub hooks: Hooks,
}
