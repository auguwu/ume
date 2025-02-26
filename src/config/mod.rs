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

use azalia::config::{env, merge::Merge, FromEnv, TryFromEnv};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use std::{
    env::VarError,
    fs,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, Merge)]
pub struct Config {
    #[merge(strategy = azalia::config::merge::strategy::strings::overwrite_empty)]
    #[serde(default)]
    pub uploader_key: String,

    #[serde(default = "__default_base_url")]
    pub base_url: Url,

    #[serde(default, skip_serializing_if = "Option::is_some")]
    pub sentry_dsn: Option<String>,

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
    url::Url::parse("http://localhost:3621").expect("failed to parse as url")
}

fn __merge_urls(url: &mut url::Url, right: url::Url) {
    if *url != right {
        *url = right;
    }
}

impl TryFromEnv for Config {
    type Output = Config;
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        Ok(Config {
            uploader_key: env!("UME_UPLOADER_KEY").unwrap_or_default(),
            sentry_dsn: env!("UME_SENTRY_DSN", optional),
            base_url: match env!("UME_BASE_URL") {
                Ok(val) => val.parse::<Url>()?,
                Err(VarError::NotPresent) => __default_base_url(),
                Err(_) => {
                    return Err(eyre!(
                        "environment variable `UME_BASE_URL` was not a valid utf-8 string"
                    ))
                }
            },

            logging: logging::Config::from_env(),
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

#[cfg(test)]
mod tests {
    use super::Config;
    use azalia::config::TryFromEnv;

    #[test]
    fn try_from_env() {
        let config = Config::try_from_env();
        assert!(config.is_ok());
    }
}
