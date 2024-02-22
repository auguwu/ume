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

use crate::config::Url;
use noelware_config::{env, merge::Merge, TryFromEnv};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Uses the gRPC protocol of [OpenTelemetry Collector]
    ///
    /// [OpenTelemetry Collector]: https://opentelemetry.io/docs/collector
    #[default]
    Grpc,

    /// Uses the HTTP protocol of [OpenTelemetry Collector]
    ///
    /// [OpenTelemetry Collector]: https://opentelemetry.io/docs/collector
    Http,
}

impl Merge for Kind {
    fn merge(&mut self, other: Self) {
        match (*self, other) {
            (Self::Grpc, Self::Grpc) | (Self::Http, Self::Http) => {}
            (Self::Grpc, Self::Http) | (Self::Http, Self::Grpc) => {
                *self = other;
            }
        }
    }
}

impl TryFromEnv for Kind {
    type Output = Kind;
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        match env!("UME_TRACING_OTEL_COLLECTOR") {
            Ok(res) => match res.as_str() {
                "grpc" | "" => Ok(Kind::Grpc),
                "http" => Ok(Kind::Http),
                out => Err(eyre!(format!("unknown otel collector kind [{out}]"))),
            },
            Err(std::env::VarError::NotPresent) => Ok(Kind::Grpc),
            Err(e) => Err(eyre::Report::from(e)),
        }
    }
}

/// Represents the configuration for using an [OpenTelemetry Collector] to report tracing
/// metadata, in return, can be exported to different software that supports it.
///
/// ## Example (gRPC)
/// ```hcl
/// tracing "opentelemetry" {
///   # default `kind` is grpc, so specifying `kind` is optional here :)
/// }
/// ```
///
/// ## Example (HTTP)
/// ```hcl
/// tracing "opentelemetry" {
///   // uses the gRPC protocol
///   kind = "http"
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

    /// Which kind of OpenTelemetry Collector we should configure for?
    #[serde(default)]
    pub kind: Kind,

    /// [`Url`][url::Url] used to connect to an available OpenTelemetry collector
    #[serde(default = "__default_url")]
    pub url: Url,
}

impl TryFromEnv for Config {
    type Output = Config;
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        Ok(Config {
            kind: Kind::try_from_env()?,
            url: env!("UME_TRACING_OTEL_COLLECTOR_URL", to: Url, or_else: __default_url()),

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
            kind: Kind::Grpc,
            url: __default_url(),
        }
    }
}

fn __default_url() -> Url {
    Url(url::Url::parse("grpc://localhost:4318").expect("a valid url to be parsed"))
}
