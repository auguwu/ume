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

pub mod otel;
pub mod sentry;

use azalia::config::{env, merge::Merge, FromEnv, TryFromEnv};
use eyre::Report;
use serde::{Deserialize, Serialize};

/// Configures the use of OpenTelemetry or Sentry to trace calls from [`tracing`]. Tracing can also
/// be disabled with `tracing = "disabled"` or not adding it into your configuration file as it is
/// the default.
///
/// ## Example (OpenTelemetry)
/// ```hcl
/// tracing "opentelemetry" {
///   labels = { "a": "b" }
///   kind   = "otel+grpc"
///   url    = "grpc://localhost:4318"
/// }
/// ```
///
/// ## Example (Sentry)
/// ```hcl
/// tracing "sentry" {
///   sample_rate = 0.7
/// }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Config {
    /// Represents the OpenTelemetry configuration, which configures ways to interact
    /// with [OpenTelemetry].
    ///
    /// The following protocols are supported:
    /// * HTTP / gRPC
    /// * Prometheus (enabled on most distributions, but might not be available)
    ///
    /// [OpenTelemetry]: https://opentelemetry.io
    OpenTelemetry(otel::Config),

    /// Configures the use of Sentry to allow tracing to be sent to a Sentry server
    /// of your choice. This will use the DSN provided by the `config.sentry_dsn`
    /// configuration key.
    Sentry(sentry::Config),

    /// Disables tracing and only represent tracing metadata in logs rather than
    /// being sent to other services.
    #[default]
    Disabled,
}

impl TryFromEnv for Config {
    type Output = Config;
    type Error = Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        match env!("UME_TRACING_BACKEND") {
            Ok(res) => match res.as_str() {
                "opentelemetry" | "otel" => Ok(Config::OpenTelemetry(otel::Config::try_from_env()?)),

                "sentry" => Ok(Config::Sentry(sentry::Config::from_env())),
                "" => Ok(Config::Disabled),
                out => Err(eyre!(format!("unknown tracing backend [{out}]"))),
            },
            Err(std::env::VarError::NotPresent) => Ok(Config::Disabled),
            Err(e) => Err(Report::from(e)),
        }
    }
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        match (self.clone(), other) {
            (Config::OpenTelemetry(mut otel), Config::OpenTelemetry(otel2)) => {
                otel.merge(otel2);
                *self = Config::OpenTelemetry(otel);
            }

            (Config::Sentry(mut sentry), Config::Sentry(sentry2)) => {
                sentry.merge(sentry2);
                *self = Config::Sentry(sentry);
            }

            (Config::Disabled, Config::Disabled) => {}
            (_, config) => {
                *self = config;
            }
        }
    }
}
