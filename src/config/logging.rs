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

use azalia::config::{
    env::{self, TryFromEnv},
    merge::Merge,
};
use serde::{Deserialize, Serialize};
use tracing::Level;

pub const LEVEL: &str = "UME_LOG_LEVEL";
pub const JSON: &str = "UME_LOG_JSON";

#[derive(Debug, Clone, Merge, Serialize, Deserialize)]
pub struct Config {
    /// Configures the log level of the Ume server's logging capabilities. The higher the level, the more verbose
    /// messages you'll get. For production environments, the default (`INFO`) is fine.
    #[serde(with = "azalia::serde::tracing")]
    #[merge(strategy = __merge_level)]
    pub level: Level,

    /// whether or not emit the log information as JSON blobs or not.
    #[serde(default)]
    #[merge(strategy = azalia::config::merge::strategy::bool::only_if_falsy)]
    pub json: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            level: __default_level(),
            json: false,
        }
    }
}

impl TryFromEnv for Config {
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self, Self::Error> {
        Ok(Config {
            level: env::try_parse_or(LEVEL, __default_level)?,
            json: env::try_parse_or_else(JSON, false)?,
        })
    }
}

fn __merge_level(level: &mut Level, other: Level) {
    if *level != other {
        *level = other;
    }
}

const fn __default_level() -> Level {
    Level::INFO
}
