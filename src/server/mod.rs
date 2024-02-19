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

mod config;
pub use config::*;

mod extract;
pub mod middleware;
mod routes;

use axum::{routing, Router};

pub fn create_router() -> Router {
    Router::new()
        .route("/heartbeat", routing::get(routes::heartbeat))
        .route("/images/upload", routing::post(routes::upload_image))
        .route("/images/:name", routing::get(routes::get_image))
        .route("/", routing::get(routes::main))
}
