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

mod cmds;
pub use cmds::*;

use indicatif::ProgressState;
use owo_colors::{OwoColorize, Stream};
use tracing::{level_filters::LevelFilter, Level};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer, Registry};

#[derive(Debug, Clone, clap::Parser)]
#[clap(
    bin_name = "ume",
    about = "🐻‍❄️💐 Easy, self-hostable, and flexible image host made in Rust",
    author = "Noel Towa <cutie@floofy.dev>",
    override_usage = "ume <COMMAND> [...ARGS]",
    arg_required_else_help = true
)]
pub struct Program {
    /// Configures the log level for all CLI-based commands. This will not configure the Ume server's
    /// log level when you run `ume server`.
    #[arg(global = true, short = 'l', long = "log-level", default_value_t = Level::INFO)]
    pub level: Level,

    /// suppress all log output even when `--log-level` is specified.
    #[arg(long, global = true, short = 'q')]
    pub quiet: bool,

    /// disables the use of progress bars in `ume screenshot` to indicate that a file
    /// is uploading to the server.
    #[arg(long, global = true, env = "UME_NO_PROGRESSBAR")]
    pub no_progress: bool,

    #[command(subcommand)]
    pub command: cmds::Cmd,
}

pub(crate) fn elapsed_subsec(state: &ProgressState, writer: &mut dyn std::fmt::Write) {
    let seconds = state.elapsed().as_secs();
    let sub_seconds = (state.elapsed().as_millis() % 1000) / 100;
    let formatted = format!("{seconds}.{sub_seconds}")
        .if_supports_color(Stream::Stderr, |x| x.fg_rgb::<134, 134, 134>())
        .to_string();

    let _ = writer.write_str(&formatted);
}

impl Program {
    pub fn init_logging(&self) {
        if !self.quiet {
            let filter = LevelFilter::from_level(self.level);
            let layer = tracing_subscriber::fmt::layer().with_target(true);

            match self.no_progress {
                true => tracing_subscriber::registry().with(layer).init(),
                false => {
                    let indicatif_layer: IndicatifLayer<Registry> = IndicatifLayer::new();

                    tracing_subscriber::registry()
                        .with(
                            layer
                                .with_writer(indicatif_layer.get_stderr_writer())
                                .with_filter(filter),
                        )
                        .init()
                }
            }
        }
    }
}
