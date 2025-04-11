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

use std::{env::VarError, fmt::Display, str::FromStr};

/// Given a <code>[`Result`]<T, [`VarError`]></code> and default value:
///
/// - In variant <code>[`Ok`]\({value}\)</code>, return the `{value}`.
/// - In variant <code>[`Err`]\([`VarError::Present`]\)</code>, return `default`.
/// - Otherwise, bail out.
pub fn env_from_result<T>(result: Result<T, VarError>, default: T) -> eyre::Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(VarError::NotPresent) => Ok(default),
        Err(VarError::NotUnicode(_)) => bail!("received non-unicode value in environment variable"),
    }
}

pub fn env_from_str<F: FromStr>(key: &str, default: F) -> eyre::Result<F>
where
    F::Err: Display,
{
    match std::env::var(key) {
        Ok(value) => value
            .parse::<F>()
            .map_err(|e| eyre!("failed to parse environment variable `${}`: {}", key, e)),

        Err(VarError::NotPresent) => Ok(default),
        Err(VarError::NotUnicode(_)) => {
            bail!("received non-unicode in environment variable `${}`", key)
        }
    }
}

pub fn bool_env(key: &str) -> eyre::Result<bool> {
    env_from_result(std::env::var(key).map(|x| azalia::TRUTHY_REGEX.is_match(&x)), false)
}
