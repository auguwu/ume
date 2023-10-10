// 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
// Copyright 2021-2023 Noel Towa <cutie@floofy.dev>
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

use crate::cli::AsyncExecute;
use async_trait::async_trait;
use eyre::Result;
use std::path::PathBuf;

/// Starts the Ume server to handle image uploading.
#[derive(Debug, Clone, clap::Parser)]
pub struct Server {
    /// amount of Tokio workers that'll be available, this will default to the amount
    /// of available CPU cores you have.
    #[arg(long, short = 'w')]
    pub workers: Option<usize>,

    /// whether or not if the configuration should be printed or not.
    #[arg(long)]
    print_config: bool,

    /// configuration path, you can use the `UME_CONFIG_PATH` environment
    /// variable to do the same.
    #[arg(long, short = 'c')]
    config: Option<PathBuf>,
}

#[async_trait]
impl AsyncExecute for Server {
    async fn execute(&self) -> Result<()> {
        Ok(())
    }
}
