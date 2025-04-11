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

use charted_core::ResultExt;
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

    color_eyre::install()?;

    let program = Program::parse();
    let runtime = (match program.command {
        Cmd::Server(ref server) => {
            let workers = cmp::max(num_cpus::get(), server.workers.unwrap_or(num_cpus::get()));
            Builder::new_multi_thread()
                .worker_threads(workers)
                .thread_name_fn(thread_name_fn)
                .enable_all()
                .build()
                .into_report()
        }

        _ => {
            program.init_logging();
            Builder::new_current_thread().enable_all().build().into_report()
        }
    })?;

    runtime.block_on(program.command.execute())
}

fn thread_name_fn() -> String {
    static ID: AtomicUsize = AtomicUsize::new(0);
    let id = ID.fetch_add(1, Ordering::SeqCst);

    format!("ume-worker-pool[#{id}]")
}
