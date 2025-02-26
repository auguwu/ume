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
use azalia::config::{env, merge::Merge, TryFromEnv};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env::VarError};

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
    type Output = Config;
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        Ok(Config {
            url: match env!("UME_TRACING_OTEL_COLLECTOR_URL") {
                Ok(val) => match val.parse::<Url>() {
                    Ok(val) => val,
                    Err(e) => return Err(eyre!("failed to parse value [{val}]: {e}")),
                },

                Err(VarError::NotPresent) => __default_url(),
                Err(_) => {
                    return Err(eyre!(
                        "received invalid utf-8 from `UME_TRACING_OTEL_COLLECTOR_URL` environment variable"
                    ))
                }
            },

            // syntax: UME_TRACING_OTEL_LABELS=key1=key2,key3=key4,key5=key6
            labels: match env!("UME_TRACING_OTEL_LABELS") {
                Ok(res) => {
                    let mut h = HashMap::new();
                    for line in res.split(',') {
                        if let Some((key, val)) = line.split_once('=') {
                            // skip if there was more than one '='
                            if val.contains('=') {
                                continue;
                            }

                            h.insert(key.to_string(), val.to_string());
                        }
                    }

                    h
                }
                Err(std::env::VarError::NotPresent) => HashMap::new(),
                Err(e) => return Err(eyre::Report::from(e)),
            },
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
    use azalia::config::{expand_with, TryFromEnv};
    use azalia::hashmap;

    #[test]
    fn test_config_without_special_env() {
        let config = Config::try_from_env();
        assert!(config.is_ok());
    }

    #[test]
    fn test_config_key_kind() {
        expand_with("UME_TRACING_OTEL_COLLECTOR", "", || {
            let config = Config::try_from_env();
            assert!(config.is_ok());
        });

        {
            expand_with("UME_TRACING_OTEL_COLLECTOR", "http", || {
                let config = Config::try_from_env();
                assert!(config.is_ok());
            });
        }
    }

    #[test]
    fn test_config_key_labels() {
        expand_with(
            "UME_TRACING_OTEL_LABELS",
            "hello=world,key1=key2;key3=key4,weow=fluff",
            || {
                let config = Config::try_from_env().unwrap();
                assert_eq!(
                    config.labels,
                    hashmap! {
                        "hello" => "world",
                        "weow" => "fluff"
                    }
                );
            },
        );
    }
}
