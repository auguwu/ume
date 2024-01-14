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

pub mod cli;
pub mod config;
pub mod logging;
pub mod metrics;
pub mod server;
pub mod storage;

use once_cell::sync::Lazy;
use rand::distributions::{Alphanumeric, DistString};
use regex::Regex;
use std::any::Any;

/// Generic [`Regex`] implementation for possible truthy boolean values.
pub static TRUTHY_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"^(yes|true|si*|e|enable|1)$"#).unwrap());

/// Returns the Rust compiler version that ume was built from
pub const RUSTC_VERSION: &str = env!("UME_RUSTC_VERSION");

/// Returns the Git commit from the canonical repository of ume.
pub const COMMIT_HASH: &str = env!("UME_COMMIT_HASH");

/// Returns the build date in the RFC3339 format of when ume was last built at.
pub const BUILD_DATE: &str = env!("UME_BUILD_DATE");

/// Returns the current version of ume.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Retrieves a panic message from a [`Box`]<dyn [`Any`]>
pub fn panic_message(error: Box<dyn Any + Send + 'static>) -> String {
    if let Some(s) = error.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = error.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(e) = error.downcast_ref::<&(dyn std::error::Error)>() {
        format!("{e}")
    } else {
        String::from("invalid panic message received")
    }
}

/// Returns a random string that is seeded by the system.
pub fn rand_string(len: usize) -> String {
    Alphanumeric.sample_string(&mut rand::thread_rng(), len)
}
