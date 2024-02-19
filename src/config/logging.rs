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

use crate::TRUTHY_REGEX;
use noelware_config::{env, merge::Merge, FromEnv};
use serde::{Deserialize, Serialize};
use tracing::Level;

#[derive(Debug, Clone, Merge, Serialize, Deserialize)]
pub struct Config {
    /// Configures the log level of the Ume server's logging capabilities. The higher the level, the more verbose
    /// messages you'll get. For production environments, the default (`INFO`) is fine.
    #[serde(with = "noelware_serde::tracing")]
    #[merge(strategy = __merge_level)]
    pub level: Level,

    /// Connection URI string to connect to a TCP server that the API server will emit log information to. This
    /// is usually with Logstash with its [`tcp` input plugin](https://www.elastic.co/guide/en/logstash/current/plugins-inputs-tcp.html) or
    /// with Noelware's [`Petal` service](https://github.com/Noelware/petal) to safely send log messages to Logstash.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logstash_tcp_uri: Option<String>,

    /// whether or not emit the log information as JSON blobs or not.
    #[serde(default)]
    #[merge(strategy = noelware_config::merge::strategy::bool::only_if_falsy)]
    pub json: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            logstash_tcp_uri: None,
            level: __default_level(),
            json: false,
        }
    }
}

impl FromEnv for Config {
    type Output = Config;

    fn from_env() -> Self::Output {
        Config {
            logstash_tcp_uri: env!("UME_LOGSTASH_TCP_URI", is_optional: true),
            json: env!("UME_LOG_JSON", {
                or_else: false;
                mapper: |val| TRUTHY_REGEX.is_match(&val);
            }),

            level: env!("UME_LOG_LEVEL", {
                or_else: __default_level();
                mapper: |val| match val.to_lowercase().as_str() {
                    "trace" => Level::TRACE,
                    "debug" => Level::DEBUG,
                    "error" => Level::ERROR,
                    "warn" => Level::WARN,
                    "info" => Level::INFO,
                    _ => __default_level(),
                };
            }),
        }
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
