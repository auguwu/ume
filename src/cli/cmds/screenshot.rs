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

use arboard::Clipboard;
use azalia::remi::core::Bytes;
use chrono::Local;
use eyre::Context;
use reqwest::multipart::{self, Part};
use serde_json::Value;
use std::{
    borrow::Cow,
    fs::{self, create_dir_all, remove_file, OpenOptions},
    io::Cursor,
    path::{Path, PathBuf},
    process::{exit, Command, Stdio},
};
use url::Url;

/// Takes a screenshot with [Flameshot](https://flameshot.org)
#[derive(Debug, Clone, clap::Parser)]
pub struct Cmd {
    /// URL that points to a Ume server
    #[arg(env = "UME_SERVER")]
    server: Url,

    /// uploader key to upload images to a Ume server
    #[arg(long = "master-key", env = "UME_UPLOADER_KEY")]
    master_key: String,

    /// directory to place all screenshots in, this will default to `$TMPDIR`.
    #[arg(long, short = 'd', env = "UME_TEMP_DIRECTORY")]
    tempdir: Option<PathBuf>,

    /// path to a `flameshot` executable, defaults to one that is found in `$PATH`
    #[arg(long, env = "FLAMESHOT")]
    flameshot: Option<PathBuf>,

    /// disables copying the image URL or the image itself (if it failed to upload) onto
    /// the system clipboard.
    #[arg(long, env = "UME_NO_COPY")]
    no_copy: bool,
}

pub async fn execute(cmd: Cmd) -> eyre::Result<()> {
    let clipboard = match arboard::Clipboard::new() {
        Ok(board) => Some(board),
        Err(e) => {
            error!(error = %e, "unable to get system clipboard; setting NOCOPY = true");
            None
        }
    };

    let tempdir = cmd.tempdir.clone().unwrap_or(std::env::temp_dir());
    let screenshots = tempdir.join("screenshots");
    if !screenshots.try_exists()? {
        create_dir_all(&screenshots)?;
    }

    let flameshot = cmd
        .flameshot
        .clone()
        .unwrap_or(which::which("flameshot").context("unable to find `flameshot` program")?);

    info!(flameshot = %flameshot.display(), "found `flameshot` program :3");
    let name = screenshots.join(format!("{}.png", Local::now().to_rfc3339()));
    info!(file = %name.display(), "creating file...");

    let file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(screenshots.join(&name))?;

    info!("$ {} gui -r > {}", flameshot.display(), name.display());

    let mut ccmd = Command::new(&flameshot);
    ccmd.args(["gui", "-r"])
        .stdout(file.try_clone()?)
        .stderr(Stdio::inherit())
        .stdin(Stdio::null());

    let output = ccmd.output()?;
    if !output.status.success() {
        error!(file = %name.display(), "failed to run `flameshot gui -r` onto file");
        remove_file(&name)?;

        exit(1);
    }

    info!(file = %name.display(), "uploading file to Ume server...");

    // upload the image
    upload_file(&cmd, &name, clipboard).await
}

async fn upload_file(cmd: &Cmd, loc: &Path, clipboard: Option<Clipboard>) -> eyre::Result<()> {
    info!(file = %loc.display(), "Now uploading file...");

    let client = reqwest::Client::builder()
        .user_agent(format!(
            "auguwu/ume+cli (+https://github.com/auguwu/ume; {}",
            crate::version()
        ))
        .build()?;

    let contents = fs::read(loc).map(Bytes::from)?;
    let res = client
        .post(format!("{}images/upload", cmd.server))
        .header("Authorization", &cmd.master_key)
        .multipart(multipart::Form::new().part("fdata", Part::stream(contents.clone())))
        .send()
        .await?;

    let status = res.status();
    let data: Value = res.json().await?;
    if !data.is_object() {
        error!("unexpected json payload from Ume server: {}", data);
        exit(1);
    }

    let obj = data.as_object().unwrap();
    if obj.contains_key("message") {
        let msg = obj["message"].as_str().unwrap();

        error!("received message from Ume server [{status}]: {msg}");
        if cmd.no_copy || clipboard.is_none() {
            return Ok(());
        }

        let mut clipboard = clipboard.unwrap();
        let img: image::DynamicImage = image::ImageReader::new(Cursor::new(&contents)).decode()?;

        clipboard.set_image(arboard::ImageData {
            height: img.height() as usize,
            width: img.width() as usize,
            bytes: Cow::Borrowed(contents.as_ref()),
        })?;

        return Ok(());
    }

    let url = obj["filename"].as_str().unwrap();
    if cmd.no_copy || clipboard.is_none() {
        eprintln!("{}", url);
        return Ok(());
    }

    info!("copying url [{url}] to clipboard!");

    let mut clipboard = clipboard.unwrap();
    #[cfg(target_os = "linux")]
    {
        use arboard::SetExtLinux;
        use std::time::{Duration, Instant};

        // wait ~1s to block
        clipboard
            .set()
            .wait_until(Instant::now() + Duration::from_secs(1))
            .text(url)?;
    }

    #[cfg(not(target_os = "linux"))]
    clipboard.set_text(url)?;

    info!("copied to clipboard, deleting image");
    fs::remove_file(loc).context("unable to delete file")
}
