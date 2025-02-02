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

use chrono::{DateTime, Utc};
use std::{process::Command, time::SystemTime};
use which::which;

fn main() {
    // if build.rs changes in any way, then re-run it!
    println!("cargo:rerun-if-changed=build.rs");

    let rust_version = rustc_version::version()
        .expect("unable to get 'rustc' version")
        .to_string();

    println!("cargo:rustc-env=UME_RUSTC_VERSION={rust_version}");

    let build_date = {
        let now = SystemTime::now();
        let date: DateTime<Utc> = now.into();

        date.to_rfc3339()
    };

    println!("cargo:rustc-env=UME_BUILD_DATE={build_date}");

    // First, we need to get the Git commit hash. There is ways we can do it:
    //      1. Use `git rev-parse --short=8 HEAD`, if `git` exists
    //      2. fuck it and ball with `d1cebae` as the dummy hash
    match which("git") {
        Ok(git) => {
            let mut cmd = Command::new(git);
            cmd.args(["rev-parse", "--short=8", "HEAD"]);

            let output = cmd.output().expect("to succeed");
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            println!("cargo:rustc-env=UME_COMMIT_HASH={stdout}");
        }

        Err(which::Error::CannotFindBinaryPath) => {
            println!("cargo:warning=missing `git` binary, using `d1cebae` as the commit hash instead");

            println!("cargo:rustc-env=UME_COMMIT_HASH=d1cebae");
        }

        Err(e) => {
            panic!("failed to get `git` from `$PATH`: {e}");
        }
    }
}
