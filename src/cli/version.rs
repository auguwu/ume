// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright (c) 2020-2023 Noel <cutie@floofy.dev>
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use anyhow::Result;
use chrono::DateTime;
use clap::Parser;

use super::types::Execute;
use crate::{BUILD_DATE, COMMIT_HASH, VERSION};

#[derive(Debug, Clone, Parser)]
pub struct Version;

impl Execute for Version {
    fn execute(&self) -> Result<()> {
        let datetime =
            DateTime::parse_from_rfc3339(BUILD_DATE)?.format("%a, %h %d, %Y at %H:%M:%S %Z");

        println!("ume v{VERSION}+{COMMIT_HASH} ({datetime})");
        Ok(())
    }
}
