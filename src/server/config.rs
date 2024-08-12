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

pub mod ssl;

use azalia::TRUTHY_REGEX;
use azalia::config::{env, merge::Merge, TryFromEnv};
use serde::{Deserialize, Serialize};
use std::{env::VarError, net::SocketAddr};

#[derive(Debug, Clone, Merge, Serialize, Deserialize)]
pub struct Config {
    /// Host to bind onto. `127.0.0.1` is for internal, `0.0.0.0` is for public.
    #[serde(default = "__default_host")]
    pub host: String,

    /// Port to listen on.
    #[serde(default = "__default_port")]
    pub port: u16,

    /// Configures the use of HTTPS on the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssl: Option<ssl::Config>,
}

impl Config {
    pub fn addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port).parse().unwrap()
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            host: __default_host(),
            port: __default_port(),
            ssl: None,
        }
    }
}

impl TryFromEnv for Config {
    type Output = Config;
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        Ok(Config {
            host: match env!("UME_SERVER_HOST") {
                Ok(val) => val,
                Err(VarError::NotPresent) => match env!("HOST") {
                    Ok(val) => val,
                    Err(VarError::NotPresent) => __default_host(),
                    Err(VarError::NotUnicode(_)) => {
                        return Err(eyre!(
                            "failed to represent `HOST` environment variable as valid unicode"
                        ))
                    }
                },

                Err(VarError::NotUnicode(_)) => {
                    return Err(eyre!(
                    "failed to represent `UME_SERVER_HOST` environment variable as valid unicode"
                ))
                }
            },

            port: match env!("UME_SERVER_PORT") {
                Ok(val) => match val.parse::<u16>() {
                    Ok(val) => val,
                    Err(e) => return Err(eyre!(e.to_string())),
                },

                Err(VarError::NotPresent) => match env!("PORT") {
                    Ok(val) => match val.parse::<u16>() {
                        Ok(val) => val,
                        Err(e) => return Err(eyre!(e.to_string())),
                    },
                    Err(VarError::NotPresent) => __default_port(),
                    Err(VarError::NotUnicode(_)) => {
                        return Err(eyre!(
                            "failed to represent `PORT` environment variable as valid unicode"
                        ))
                    }
                },

                Err(VarError::NotUnicode(_)) => {
                    return Err(eyre!(
                    "failed to represent `UME_SERVER_PORT` environment variable as valid unicode"
                ))
                }
            },

            ssl: match env!("UME_SERVER_SSL_ENABLE") {
                Ok(res) if TRUTHY_REGEX.is_match(&res) => Some(ssl::Config::try_from_env()?),
                Ok(_) => None,

                Err(std::env::VarError::NotUnicode(_)) => {
                    return Err(eyre!("expected valid utf-8 for `UME_SERVER_SSL_ENABLE`"));
                }

                Err(_) => None,
            },
        })
    }
}

#[inline]
fn __default_host() -> String {
    String::from("0.0.0.0")
}

#[inline]
fn __default_port() -> u16 {
    3621
}
