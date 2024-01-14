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

pub mod commands;

use async_trait::async_trait;
use eyre::Result;

#[async_trait]
pub trait AsyncExecute {
    async fn execute(&self) -> Result<()>;
}

#[derive(Debug, Clone, clap::Parser)]
#[clap(
    bin_name = "ume",
    about = "🐻‍❄️💐 Easy, self-hostable, and flexible image host made in Rust",
    author = "Noel Towa <cutie@floofy.dev>",
    override_usage = "ume <COMMAND> [...ARGS]",
    arg_required_else_help = true
)]
pub struct Program {
    /// whether or not verbose mode is enabled
    #[arg(long, short = 'v', env = "UME_VERBOSE")]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: commands::Cmd,
}

#[async_trait]
impl AsyncExecute for Program {
    async fn execute(&self) -> Result<()> {
        match &self.command {
            commands::Cmd::Server(server) => server.execute().await,
            _ => Ok(()),
        }
    }
}
