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

mod middleware;
pub mod routing;

use crate::{config::Config, storage::StorageService};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Represents a Ume server, which is the main HTTP interface.
#[derive(Debug)]
pub struct Server {
    pub storage: StorageService,
    pub requests: AtomicUsize,
    pub config: Config,
}

impl Clone for Server {
    fn clone(&self) -> Server {
        Server {
            requests: AtomicUsize::new(self.requests.load(Ordering::Relaxed)),
            storage: self.storage.clone(),
            config: self.config.clone(),
        }
    }
}
