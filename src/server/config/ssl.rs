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

use eyre::Context;
use azalia::config::{env, merge::Merge, TryFromEnv};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Merge, Serialize, Deserialize)]
pub struct Config {
    /// Location to a certificate private key.
    #[merge(skip)]
    pub cert_key: PathBuf,

    /// Location to a certificate public key.
    #[merge(skip)]
    pub cert: PathBuf,
}

impl Default for Config {
    fn default() -> Config {
        let certs = PathBuf::from("./certs");
        Config {
            cert_key: certs.join("key.pem"),
            cert: certs.join("cert.pem"),
        }
    }
}

impl TryFromEnv for Config {
    type Output = Config;
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        Ok(Config {
            cert_key: env!("UME_SERVER_SSL_CERT_KEY")
                .map(PathBuf::from)
                .context("unable to load up `UME_SERVER_SSL_CERT_KEY` env")?,

            cert: env!("UME_SERVER_SSL_CERT")
                .map(PathBuf::from)
                .context("unable to load up `UME_SERVER_SSL_CERT` env")?,
        })
    }
}
