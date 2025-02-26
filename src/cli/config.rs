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

pub mod system_notifications;
pub mod uploader;

use etcetera::{base_strategy::choose_native_strategy, BaseStrategy};
use eyre::Context;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tracing::warn;

/// Configuration for the `ume` CLI commands, like `ume screenshot`.
///
/// As of **v4.1**, a configuration file will be initialized in
/// `$CONFIG_DIR/Noel/ume/config.toml` where commands like `ume screenshot`
/// can reference uploaders.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    /// A list of uploaders that CLI commands like `ume screenshot` can use.
    #[serde(default, rename = "uploader", skip_serializing_if = "Vec::is_empty")]
    pub uploaders: Vec<uploader::Config>,

    #[cfg(feature = "os-notifier")]
    #[serde(default)]
    /// whether if CLI commands can use the system's notifications for sending messages.
    pub system_notifications: system_notifications::Config,
}

impl Config {
    pub fn load<P: Into<Option<PathBuf>>>(path: P) -> eyre::Result<Self> {
        let Some(path) = path.into() else {
            use std::io::Write;

            let path = choose_native_strategy().map(|s| s.config_dir().join("Noel/ume/config.toml"))?;
            if !path.try_exists()? {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let config = Config::default();
                let serialized = toml::to_string_pretty(&config).unwrap();

                let mut file = fs::File::create_new(path)?;
                write!(file, "{serialized}")?;

                return Ok(config);
            }

            return Config::load(Some(path));
        };

        if !path.try_exists()? {
            warn!(
                "unable to find configuration file in {}, using default configuration",
                path.display()
            );

            return Ok(Config::default());
        }

        info!(path = %path.display(), "loading CLI configuration...");
        let contents = fs::read_to_string(&path)?;
        toml::from_str(&contents).with_context(|| format!("failed to deserialize file {}", path.display()))
    }
}
