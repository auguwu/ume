// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright 2021-2025 Noel Towa <cutie@floofy.dev>
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

pub mod logging;
pub mod storage;
pub mod tracing;
pub mod util;

use azalia::config::{
    env::{self, TryFromEnv},
    merge::Merge,
};
use rand::distr::{Alphanumeric, SampleString};
use sentry::types::Dsn;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;

const UPLOADER_KEY: &str = "UME_UPLOADER_KEY";
const SENTRY_DSN: &str = "UME_SENTRY_DSN";
const BASE_URL: &str = "UME_BASE_URL";

#[derive(Debug, Clone, Serialize, Deserialize, Merge)]
pub struct Config {
    #[merge(strategy = azalia::config::merge::strategy::strings::overwrite_empty)]
    #[serde(default)]
    pub uploader_key: String,

    #[serde(default = "__default_base_url")]
    pub base_url: Url,

    #[serde(default, skip_serializing_if = "Option::is_some")]
    pub sentry_dsn: Option<Dsn>,

    #[serde(default)]
    pub logging: logging::Config,

    #[serde(default)]
    pub storage: storage::Config,

    #[serde(default)]
    pub tracing: tracing::Config,

    #[serde(default)]
    pub server: crate::server::Config,
}

fn __default_base_url() -> Url {
    Url::parse("http://localhost:3621").expect("failed to parse as url")
}

impl TryFromEnv for Config {
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self, Self::Error> {
        Ok(Config {
            uploader_key: env::try_parse(UPLOADER_KEY).unwrap_or_default(),
            sentry_dsn: env::try_parse_optional(SENTRY_DSN)?,
            base_url: env::try_parse_or(BASE_URL, __default_base_url)?,

            logging: logging::Config::try_from_env()?,
            storage: storage::Config::try_from_env()?,
            tracing: tracing::Config::try_from_env()?,
            server: crate::server::Config::try_from_env()?,
        })
    }
}

impl Config {
    pub fn find_default_location() -> Option<PathBuf> {
        let config_path = PathBuf::from("./config/ume.hcl");
        if config_path.exists() {
            return Some(config_path);
        }

        let config_path = PathBuf::from("./config/ume.toml");
        if config_path.exists() {
            return Some(config_path);
        }

        match std::env::var("UME_CONFIG_FILE") {
            Ok(path) => {
                let path = Path::new(&path);
                if path.exists() && path.is_file() {
                    return Some(path.to_path_buf());
                }

                None
            }

            Err(_) => {
                let last_resort = Path::new("./config.hcl");
                if last_resort.exists() && last_resort.is_file() {
                    return Some(last_resort.to_path_buf());
                }

                let last_resort = Path::new("./config.toml");
                if last_resort.exists() && last_resort.is_file() {
                    return Some(last_resort.to_path_buf());
                }

                None
            }
        }
    }

    /// Creates a new [`Config`] instance from a given path.
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> eyre::Result<Config> {
        // priority: config file > env variables
        let Some(path) = path.as_ref() else {
            return Config::try_from_env();
        };

        let path = path.as_ref();
        if !path.try_exists()? {
            eprintln!(
                "[ume WARN] file {} doesn't exist, using system env variables",
                path.display()
            );

            return Config::try_from_env();
        }

        let mut cfg = Config::try_from_env()?;
        let file = toml::from_str(&fs::read_to_string(path)?)?;

        cfg.merge(file);

        if cfg.uploader_key.is_empty() {
            let key = __generated_uploader_key();
            eprintln!("[ume WARN] Missing a uploader key for authentication! I have generated one for you:\n
\t\t{key}\n
Set this in the `UME_UPLOADER_KEY` environment variable when loading the server or in the `uploader_key` in your `config.hcl` file.
If any other key replaces this, then it'll no longer be verified. It is recommended to keep this safe somewhere");

            cfg.uploader_key = key;
        }

        Ok(cfg)
    }
}

fn __generated_uploader_key() -> String {
    Alphanumeric.sample_string(&mut rand::rng(), 32)
}

macro_rules! impl_enum_based_env_value {
    ($env:expr, {
        on match fail: |$input:ident| $error:literal$( [$($arg:expr),*])?;

        $($field:pat => $value:expr;)*
    }) => {
        match ::azalia::config::env::try_parse_or_else::<_, ::std::string::String>($env, ::core::default::Default::default()) {
            Ok($input) => match &*$input.to_ascii_lowercase() {
                $($field => $value,)*

                #[allow(unreachable_patterns)]
                _ => ::eyre::bail!($error$(, $($arg),*)?)
            }

            Err(::azalia::config::env::TryParseError::System(::std::env::VarError::NotUnicode(_))) => {
                ::eyre::bail!("environment variable `${}` couldn't be loaded due to invalid unicode data", $env)
            }

            Err(e) => return Err(::core::convert::Into::into(e)),
        }
    };

    ($env:expr, {
        on match fail: $other:expr;

        $($field:pat => $value:expr;)*
    }) => {
        match ::azalia::config::env::try_parse_or_else::<_, ::std::string::String>($env, ::core::default::Default::default()) {
            Ok(input) => match &*input.to_ascii_lowercase() {
                $($field => $value,)*

                #[allow(unreachable_patterns)]
                _ => ::core::result::Result::Ok($other)
            }

            Err(::azalia::config::env::TryParseError::System(::std::env::VarError::NotUnicode(_))) => {
                ::eyre::bail!("environment variable `${}` couldn't be loaded due to invalid unicode data", $env)
            }

            Err(e) => Err(::core::convert::Into::into(e)),
        }
    };
}

pub(crate) use impl_enum_based_env_value;

#[cfg(test)]
mod tests {
    use super::Config;
    use azalia::config::env::TryFromEnv;

    #[test]
    fn try_from_env() {
        let config = Config::try_from_env();
        assert!(config.is_ok());
    }
}
