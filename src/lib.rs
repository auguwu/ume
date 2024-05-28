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

use std::sync::OnceLock;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate eyre;

pub mod cli;
pub mod config;
pub mod server;

/// Constant that refers to the version of the Rust compiler that was used. This is mainly
/// for diagnostics and is never accessed by third parties.
pub const RUSTC: &str = env!("UME_RUSTC_VERSION");

/// Constant in the format of [RFC3339] date format that refers to when `ume` was last built
///
/// [RFC3339]: https://www.rfc-editor.org/rfc/rfc3339
pub const BUILD_DATE: &str = env!("UME_BUILD_DATE");

/// Constant that refers to the Git commit hash from the [canonical repository]
///
/// [canonical repository]: https://github.com/auguwu/ume
pub const COMMIT_HASH: &str = env!("UME_COMMIT_HASH");

/// Constant that refers to the version of the `ume` software
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns a formatted version of `v4.0.0+d1cebae` or `v4.0.0` if no commit hash
/// was found.
///
/// This will return a immutable string slice as the version, and since it could possibly
/// be mutated, we advise to only use it in immutable contexts; never try to mutate it.
#[inline]
pub fn version() -> &'static str {
    static ONCE: OnceLock<String> = OnceLock::new();
    ONCE.get_or_init(|| {
        use std::fmt::Write;

        let mut buf = String::new();
        write!(buf, "{}", crate::VERSION).unwrap();

        if crate::COMMIT_HASH != "d1cebae" {
            write!(buf, "+{}", crate::COMMIT_HASH).unwrap();
        }

        buf
    })
}
