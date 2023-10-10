// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright 2021-2023 Noel Towa <cutie@floofy.dev>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{panic_message, storage::config::StorageConfig, var};
use eyre::{eyre, Result};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::{
    env::VarError,
    panic::catch_unwind,
    path::{Path, PathBuf},
};
use tracing::warn;

mod from_env;
mod merge;
mod server;

pub use from_env::*;
pub use merge::*;
pub use server::*;

static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// Master key to allow access to upload images.
    pub master_key: String,

    /// Sentry [DSN](https://docs.sentry.io/product/sentry-basics/concepts/dsn-explainer/) to apply when configuring
    /// Sentry to report Rust errors if any occur.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sentry_dsn: Option<String>,

    /// Storage configuration to hold all images.
    #[serde(default, with = "serde_yaml::with::singleton_map")]
    pub storage: StorageConfig,
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        self.sentry_dsn.merge(other.sentry_dsn);

        // master key and storage don't need to be merged
    }
}

impl FromEnv for Config {
    type Output = Config;

    fn from_env() -> Self::Output {
        Config {
            master_key: var!("UME_MASTER_KEY").unwrap_or_default(),
            sentry_dsn: var!("UME_SENTRY_DSN", is_optional: true),
            storage: StorageConfig::from_env(),
        }
    }
}

impl Config {
    fn find_from_default_location() -> Result<Option<PathBuf>> {
        // 1. ./config/ume.yaml
        let config_dir = PathBuf::from("./config");
        let ume_yaml = config_dir.join("ume.yaml");

        if config_dir.try_exists()?
            && config_dir.is_dir()
            && ume_yaml.try_exists()?
            && ume_yaml.is_file()
        {
            return Ok(Some(ume_yaml));
        }

        // 2. ./config.yml | config.yaml
        let config_yml = PathBuf::from("./config.yml");
        if config_yml.try_exists()? && config_yml.is_file() {
            return Ok(Some(config_yml));
        }

        let config_yaml = PathBuf::from("./config.yaml");
        if config_yaml.try_exists()? && config_yaml.is_file() {
            return Ok(Some(config_yaml));
        }

        // 3. UME_CONFIG_PATH environment variable
        match var!("UME_CONFIG_PATH", to: PathBuf) {
            Ok(path) if path.try_exists()? && path.is_file() => Ok(Some(path)),
            Ok(_) => Ok(None),
            Err(e) => match e {
                VarError::NotPresent => Ok(None),
                VarError::NotUnicode(_) => {
                    Err(eyre!("`UME_CONFIG_PATH` received invalid unicode data"))
                }
            },
        }
    }

    /// Returns the loaded [`Config`] reference. This method will panic if
    /// there is no configuration loaded via [`Config::load`].
    pub fn get<'a>(&self) -> &'a Config {
        CONFIG.get().unwrap()
    }

    /// Loads the configuration in the given `path` if there is a path available. If not,
    /// it'll try to locate in:
    ///
    /// * `./config/ume.yaml`
    /// * `./config.yml` or `./config.yaml`
    /// * `UME_CONFIG_PATH` environment variable
    pub fn load<P: AsRef<Path>>(path: Option<P>) -> Result<()> {
        // fast path if we already have a loaded config
        if CONFIG.get().is_some() {
            warn!("configuration was already loaded!");
            return Ok(());
        }

        let path = match path {
            Some(p) => p.as_ref().to_path_buf(),
            None => match Config::find_from_default_location() {
                Ok(Some(p)) => p,

                // last resort is system env
                Ok(None) => {
                    let mut default = Config::default();
                    let config = catch_unwind(Config::from_env).map_err(|e| {
                        eyre!(
                            "unable to transform from system env variables to ume configuration: {}",
                            panic_message(e)
                        )
                    })?;

                    Config::merge(&mut default, config);
                    CONFIG.set(default).unwrap();

                    return Ok(());
                }

                Err(e) => {
                    return Err(eyre!(
                        "unable to find compatible configuration file due to [{e}]"
                    ))
                }
            },
        };

        let mut default = Config::default();
        let from_env = catch_unwind(Config::from_env).map_err(|e| {
            eyre!(
                "unable to transform from system env variables to ume configuration: {}",
                panic_message(e)
            )
        })?;

        let from_file = serde_yaml::from_reader::<_, Config>(std::fs::File::open(path)?)?;

        // merge defaults and environment variables
        Config::merge(&mut default, from_env);

        // merge default from the on-file configuration
        Config::merge(&mut default, from_file);

        CONFIG.set(default).unwrap();
        Ok(())
    }
}

// stolen from https://github.com/charted-dev/charted/blob/main/crates/config/src/lib.rs
/// Generic Rust functional macro to help with locating an environment variable
/// in the host machine.
///
/// ## Variants
/// ### `var!($key: literal)`
/// This will just expand `$key` into a Result<[`String`][alloc::string::String], [`VarError`][std::env::VarError]> variant.
///
/// ```
/// # use ume::var;
/// #
/// let result = var!("SOME_ENV_VARIABLE");
/// // expanded: ::std::env::var("SOME_ENV_VARIABLE");
/// #
/// # assert!(result.is_err());
/// ```
///
/// ### `var!($key: literal, is_optional: true)`
/// Expands the `$key` into a Option type if a [`VarError`][std::env::VarError] occurs.
///
/// ```
/// # use ume::var;
/// #
/// let result = var!("SOME_ENV_VARIABLE", is_optional: true);
/// // expanded: ::std::env::var("SOME_ENV_VARIABLE").ok();
/// #
/// # assert!(result.is_none());
/// ```
///
/// ### `var!($key: literal, or_else: $else: expr)`
/// Expands `$key` into a String, but if a [`VarError`][std::env::VarError] occurs, then a provided `$else`
/// is used as the default.
///
/// ```
/// # use ume::var;
/// #
/// let result = var!("SOME_ENV_VARIABLE", or_else: "".into());
/// // expanded: ::std::env::var("SOME_ENV_VARIABLE").unwrap_or("".into());
/// #
/// # assert!(result.is_empty());
/// ```
///
/// ### `var!($key: literal, or_else_do: $else: expr)`
/// Same as [`var!($key: literal, or_else: $else: expr)`][crate::var], but uses `.unwrap_or_else` to
/// accept a [`Fn`][std::ops::Fn].
///
/// ```
/// # use ume::var;
/// #
/// let result = var!("SOME_ENV_VARIABLE", or_else_do: |_| Default::default());
/// // expanded: ::std::env::var("SOME_ENV_VARIABLE").unwrap_or_else(|_| Default::default());
/// #
/// # assert!(result.is_empty());
/// ```
///
/// ### `var!($key: literal, use_default: true)`
/// Same as [`var!($key: literal, or_else_do: $else: expr)`][crate::var], but will use the
/// [Default][core::default::Default] implementation, if it can be resolved.
///
/// ```
/// # use ume::var;
/// #
/// let result = var!("SOME_ENV_VARIABLE", use_default: true);
/// // expanded: ::std::env::var("SOME_ENV_VARIABLE").unwrap_or_else(|_| Default::default());
/// #
/// # assert!(result.is_empty());
/// ```
///
/// ### `var!($key: literal, mapper: $mapper: expr)`
/// Uses the [`.map`][result-map] method with an accepted `mapper` to map to a different type.
///
/// ```
/// # use ume::var;
/// #
/// let result = var!("SOME_ENV_VARIABLE", mapper: |val| &val == "true");
///
/// /*
/// expanded:
/// ::std::env::var("SOME_ENV_VARIABLE").map(|val| &val == "true");
/// */
/// #
/// # assert!(result.is_err());
/// ```
///
/// [result-map]: https://doc.rust-lang.org/nightly/core/result/enum.Result.html#method.map
#[macro_export]
macro_rules! var {
    ($key:literal, to: $ty:ty, or_else: $else_:expr) => {
        var!($key, mapper: |p| {
            p.parse::<$ty>().expect(concat!(
                "Unable to resolve env var [",
                $key,
                "] to a [",
                stringify!($ty),
                "] value"
            ))
        })
        .unwrap_or($else_)
    };

    ($key:literal, to: $ty:ty, is_optional: true) => {
        var!($key, mapper: |p| p.parse::<$ty>().ok()).unwrap_or(None)
    };

    ($key:literal, to: $ty:ty) => {
        var!($key, mapper: |p| {
            p.parse::<$ty>().expect(concat!(
                "Unable to resolve env var [",
                $key,
                "] to a [",
                stringify!($ty),
                "] value"
            ))
        })
    };

    ($key:literal, {
        or_else: $else_:expr;
        mapper: $mapper:expr;
    }) => {
        var!($key, mapper: $mapper).unwrap_or($else_)
    };

    ($key:literal, mapper: $expr:expr) => {
        var!($key).map($expr)
    };

    ($key:literal, use_default: true) => {
        var!($key, or_else_do: |_| Default::default())
    };

    ($key:literal, or_else_do: $expr:expr) => {
        var!($key).unwrap_or_else($expr)
    };

    ($key:literal, or_else: $else_:expr) => {
        var!($key).unwrap_or($else_)
    };

    ($key:literal, is_optional: true) => {
        var!($key).ok()
    };

    ($key:literal) => {
        ::std::env::var($key)
    };
}
