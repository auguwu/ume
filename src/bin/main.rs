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

use clap::Parser;
use color_eyre::config::HookBuilder;
use eyre::Result;
use num_cpus::get as cpus;
use tokio::runtime::Builder;
use ume::{
    cli::{commands::Cmd, AsyncExecute, Program},
    var, COMMIT_HASH, RUSTC_VERSION, VERSION,
};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// When `ume server` is invoked, we don't want to specify `TOKIO_WORKER_THREADS` as I want
// all env variables to configure ume itself using the `UME_` prefix, it's just personal
// preference.
//
// Otherwise, the single thread scheduler is used for CLI commands.
fn main() -> Result<()> {
    let program = Program::parse();
    if matches!(program.command, Cmd::Server(_)) && var!("TOKIO_WORKER_THREADS").is_ok() {
        eprintln!("WARN: using `TOKIO_WORKER_THREADS` won't do anything! use `UME_WORKER_THREADS` instead");
        std::env::remove_var("TOKIO_WORKER_THREADS");
    }

    // configure tokio runtime
    let runtime = match program.command {
        Cmd::Server(ref server) => {
            let workers =
                var!("UME_WORKER_THREADS", to: usize, or_else: server.workers.unwrap_or(cpus()));

            if program.verbose {
                eprintln!("[VERBOSE] using {workers} Tokio workers");
            }

            color_eyre::install()?;
            Builder::new_multi_thread()
                .worker_threads(workers)
                .thread_name("ume-tokio-worker-pool")
                .enable_all()
                .build()?
        }

        _ => {
            HookBuilder::new()
                .issue_url("https://github.com/auguwu/ume/issues/new")
                .add_issue_metadata("version", format!("v{VERSION}+{COMMIT_HASH}"))
                .add_issue_metadata("rustc", RUSTC_VERSION)
                .install()?;

            Builder::new_current_thread()
                .worker_threads(1)
                .thread_name("cli-worker-pool")
                .enable_io()
                .build()?
        }
    };

    runtime.block_on(async { program.execute().await })
}
