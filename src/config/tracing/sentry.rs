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
use std::convert::Infallible;

pub const SAMPLE_SET: &str = "UME_TRACING_SENTRY_SAMPLE_SET";

#[derive(Debug, Clone, Default, Serialize, Deserialize, Merge)]
pub struct Config {
    #[serde(default = "__default_sample_set")]
    pub sample_set: f32,
}

impl TryFromEnv for Config {
    type Error = Infallible;

    fn try_from_env() -> Result<Self, Self::Error> {
        Ok(Config {
            sample_set: env::try_parse_or(SAMPLE_SET, __default_sample_set).unwrap(),
        })
    }
}

// compute 75% of all trace sample
const fn __default_sample_set() -> f32 {
    0.75
}
