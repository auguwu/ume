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

use serde_json::json;
use std::{
    io::{self, Write as _},
    path::PathBuf,
};
use url::Url;

/// Version of the ShareX's custom uploader configuration
#[allow(dead_code)] // it is used but lint is wrong
const SHAREX_VERSION: &str = "15.0.0";

/// Generates a ShareX compatible configuration to use Ume as a image uploader on [ShareX](https://sharex.com).
#[derive(Debug, Clone, clap::Parser)]
pub struct Cmd {
    /// URI location to a Ume server, which will act as the URL to upload images from.
    #[arg(env = "UME_SERVER")]
    server: Url,

    /// Uploader key to act as authorization
    #[arg(env = "UME_UPLOADER_KEY")]
    uploader_key: String,

    /// optional file to place the contents in, this will default to stdout so it can be
    /// easily piped into.
    #[arg(long = "in", env = "UME_SHAREX_FILE")]
    in_: Option<PathBuf>,

    /// sets a custom error message for when the uploader has failed to upload a image to
    /// the server. You can use the `{json:message}` key to see why it failed.
    #[arg(long, short = 'e', env = "UME_ERROR_MESSAGE")]
    error_message: Option<String>,
}

pub fn execute(cmd: Cmd) -> eyre::Result<()> {
    let contents = json!({
        "Version": SHAREX_VERSION,
        "Name": "ume",
        "DestinationType": "ImageUploader, FileUploader",
        "RequestMethod": "POST",
        "RequestURL": format!("{}/images/upload", cmd.server),
        "Body": "MultipartFormData",
        "FileFormName": "fdata",
        "URL": format!("{}/images/{{json:filename}}", cmd.server),
        "ThumbnailURL": format!("{}/images/{{json:filename}}", cmd.server),
        "DeletionURL": null,
        "ErrorMessage": cmd.error_message.unwrap_or(format!("failed to upload to {}: {{json:message}}", cmd.server)),
        "Headers": json!({
            "Authorization": format!("Uploader {}", cmd.uploader_key)
        })
    });

    let mut stdout = io::stdout().lock();
    write!(stdout, "{}", serde_json::to_string(&contents)?)?;

    Ok(())
}
