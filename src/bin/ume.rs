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

use clap::Parser;
use std::{
    cmp,
    sync::atomic::{AtomicUsize, Ordering},
};
use tokio::runtime::Builder;
use ume::cli::{Cmd, Program};

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> eyre::Result<()> {
    dotenvy::dotenv().unwrap_or_default();

    let program = Program::parse();
    let runtime = match program.command {
        Cmd::Server(ref server) => {
            color_eyre::install()?;

            let workers = cmp::max(num_cpus::get(), server.workers.unwrap_or(num_cpus::get()));
            Builder::new_multi_thread()
                .worker_threads(workers)
                .thread_name_fn(|| {
                    static ID: AtomicUsize = AtomicUsize::new(0);
                    let id = ID.fetch_add(1, Ordering::SeqCst);

                    format!("ume-worker-pool[#{id}]")
                })
                .enable_all()
                .build()
        }

        _ => {
            color_eyre::config::HookBuilder::new()
                .issue_url("https://github.com/auguwu/ume/issues/new")
                .add_issue_metadata("version", ume::VERSION)
                .add_issue_metadata("rustc", ume::RUSTC)
                .install()?;

            program.init_logging();
            Builder::new_current_thread().worker_threads(1).enable_all().build()
        }
    }?;

    runtime.block_on(program.command.execute())
}
