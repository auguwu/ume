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

use crate::config::Url;
use azalia::config::{
    env::{self, TryFromEnv},
    merge::Merge,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const LABELS: &str = "UME_TRACING_OTEL_LABELS";
pub const URL: &str = "UME_TRACING_OTEL_URL";

/// Represents the configuration for using an [OpenTelemetry Collector] to report tracing
/// metadata, in return, can be exported to different software that supports it.
///
/// ## Example (gRPC)
/// ```toml
/// [tracing.opentelemetry]
/// url = "grpc://localhost:4318"
/// ```
///
/// ## Example (HTTP)
/// ```hcl
/// tracing "opentelemetry" {
///   url = "http://localhost:4318"
/// }
/// ```
///
/// [OpenTelemetry Collector]: https://opentelemetry.io/docs/collector
#[derive(Debug, Clone, Serialize, Deserialize, Merge)]
pub struct Config {
    /// list of labels to append when creating the connection to OpenTelemetry collector. `ume` will
    /// provide the following labels:
    ///
    /// * `service.name`
    /// * `ume.version`
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub labels: HashMap<String, String>,

    /// [`Url`][url::Url] used to connect to an available OpenTelemetry collector
    #[serde(default = "__default_url")]
    pub url: Url,
}

impl TryFromEnv for Config {
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self, Self::Error> {
        Ok(Config {
            url: env::try_parse_or(URL, __default_url)?,
            labels: env::try_parse_or(LABELS, Default::default)?,
        })
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            labels: HashMap::new(),
            url: __default_url(),
        }
    }
}

fn __default_url() -> Url {
    url::Url::parse("grpc://localhost:4318").expect("a valid url to be parsed")
}

#[cfg(test)]
mod tests {
    use super::Config;
    use azalia::config::env::TryFromEnv;

    #[test]
    fn test_config_without_special_env() {
        let config = Config::try_from_env();
        assert!(config.is_ok());
    }
}
