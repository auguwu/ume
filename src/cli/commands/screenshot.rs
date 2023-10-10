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

/// Generates a screenshot with Flameshot and uploads it to a Ume server.
#[derive(Debug, Clone, clap::Parser)]
pub struct Screenshot {
    /// master key to unlock authorization with the Ume server
    #[arg(long = "master-key", short = 'k', env = "UME_MASTER_KEY")]
    master_key: String,

    /// the url to a Ume server
    #[arg(long = "server-url", short = 's', env = "UME_SERVER_URL")]
    server: Option<String>,
}
