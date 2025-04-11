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
use eyre::eyre;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CERT_KEY: &str = "UME_SERVER_SSL_CERT_KEY";
pub const ENABLED: &str = "UME_SERVER_SSL";
pub const CERT: &str = "UME_SERVER_SSL_CERTIFICATE";

#[derive(Debug, Clone, Merge, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Location to a certificate private key.
    pub cert_key: PathBuf,

    /// Location to a certificate public key.
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
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self, Self::Error> {
        Ok(Config {
            cert_key: env::try_parse(CERT_KEY).map_err(|err| eyre!("unable to load `${}`: {}", CERT_KEY, err))?,
            cert: env::try_parse(CERT).map_err(|err| eyre!("unable to load `${}`: {}", CERT, err))?,
        })
    }
}
