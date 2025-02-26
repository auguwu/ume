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

mod cmds;
pub use cmds::*;

pub mod config;

use azalia::log::writers;
use either::Either;
use std::{io, str::FromStr};
use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

#[derive(Debug, Clone, clap::Parser)]
#[clap(
    bin_name = "ume",
    about = "🐻‍❄️💐 Easy, self-hostable, and flexible image host made in Rust",
    author = "Noel Towa <cutie@floofy.dev>",
    override_usage = "ume <COMMAND> [...ARGS]",
    arg_required_else_help = true,
    version = crate::version()
)]
pub struct Program {
    /// Configures the log level for all CLI-based commands. This will not configure the Ume server's
    /// log level when you run `ume server`.
    #[arg(global = true, short = 'l', long = "log-level", default_value_t = Level::INFO)]
    pub level: Level,

    /// suppress all log output even when `--log-level` is specified.
    #[arg(long, global = true, short = 'q')]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: cmds::Cmd,
}

impl Program {
    pub fn init_logging(&self) {
        if !self.quiet {
            tracing_subscriber::registry()
                .with(
                    azalia::log::WriteLayer::new_with(
                        io::stdout(),
                        writers::default::Writer {
                            print_thread: false,
                            print_module: false,

                            ..Default::default()
                        },
                    )
                    .with_filter(LevelFilter::from_level(self.level)),
                )
                .init();
        }
    }
}

// credit to:
// https://github.com/TheAwiteb/lprs/blob/60e0812dd2d1af93a9143b17039bd7995840c9ed/src/clap_parsers.rs#L39-L61
pub(crate) fn parse_either<L: FromStr, R: FromStr>(value: &str) -> eyre::Result<Either<L, R>>
where
    <L as FromStr>::Err: std::error::Error + Send + Sync + 'static,
    <R as FromStr>::Err: std::error::Error + Send + Sync + 'static,
{
    value
        .trim()
        .parse::<L>()
        .map_err(eyre::Report::new)
        .map(Either::Left)
        .or_else(|_| value.parse::<R>().map_err(eyre::Report::new).map(Either::Right))
}
