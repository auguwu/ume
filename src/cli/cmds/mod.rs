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

mod completions;
mod screenshot;
mod server;

#[cfg(windows)]
mod sharex;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Cmd {
    Completions(completions::Cmd),
    Screenshot(screenshot::Cmd),
    Server(server::Cmd),

    #[cfg(windows)]
    #[command(name = "sharex")]
    ShareX(sharex::Cmd),
}

impl Cmd {
    pub async fn execute(self) -> eyre::Result<()> {
        match self {
            Cmd::Completions(cmd) => completions::execute(cmd),
            Cmd::Screenshot(cmd) => screenshot::execute(cmd).await,
            Cmd::Server(cmd) => server::execute(cmd).await,

            #[cfg(windows)]
            Cmd::ShareX(cmd) => sharex::execute(cmd),
        }
    }
}
