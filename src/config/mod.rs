// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright 2021-2024 Noel Towa <cutie@floofy.dev>
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

use eyre::Context;
use noelware_config::{env, merge::Merge, TryFromEnv};
use rand::distributions::{Alphanumeric, DistString};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};

#[derive(Debug, Clone, Serialize, Deserialize, Merge)]
pub struct Config {
    #[serde(default)]
    pub uploader_key: String,

    #[serde(default = "__default_base_url")]
    #[merge(strategy = __merge_urls)]
    pub base_url: url::Url,

    #[serde(default, skip_serializing_if = "Option::is_some")]
    pub sentry_dsn: Option<String>,

    #[serde(default, serialize_with = "hcl::ser::block")]
    pub logging: logging::Config,

    #[serde(default, serialize_with = "hcl::ser::labeled_block")]
    pub storage: storage::Config,

    #[serde(default, serialize_with = "hcl::ser::labeled_block")]
    pub tracing: tracing::Config,

    #[serde(default, serialize_with = "hcl::ser::block")]
    pub server: crate::server::Config,
}

fn __default_base_url() -> url::Url {
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
            uploader_key: env!("UME_UPLOADER_KEY", or_else: String::new()),
            sentry_dsn: env!("UME_SENTRY_DSN", is_optional: true),
            base_url: env!("UME_BASE_URL", to: url::Url, or_else: __default_base_url()),
            logging: logging::Config::try_from_env().context("this should never happen")?,
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
        let file = hcl::from_reader::<Config, _>(File::open(path)?)?;

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
    Alphanumeric.sample_string(&mut rand::thread_rng(), 32)
}

/// Represents a [`url::Url`] wrapper which implements [`Merge`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url(url::Url);
impl PartialEq for Url {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Merge for Url {
    fn merge(&mut self, other: Self) {
        if self.0 != other.0 {
            *self = Url(other.0);
        }
    }
}

impl Deref for Url {
    type Target = url::Url;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Url {
    type Err = url::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        url::Url::from_str(s).map(Url)
    }
}
