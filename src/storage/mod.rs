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

pub mod config;
pub mod gridfs;

use async_trait::async_trait;
use bytes::Bytes;
use remi_core::{Blob, ListBlobsRequest, StorageService as RemiStorageService, UploadRequest};
use remi_fs::FilesystemStorageService;
use remi_s3::S3StorageService;
use std::{io::Result, path::Path};

#[derive(Debug, Clone)]
pub enum StorageService {
    Filesystem(FilesystemStorageService),
    Gridfs(gridfs::GridfsStorageService),
    S3(S3StorageService),
}

#[async_trait]
impl RemiStorageService for StorageService {
    fn name(self) -> &'static str {
        "ume:remi"
    }

    async fn init(&self) -> Result<()> {
        match self {
            StorageService::Filesystem(fs) => fs.init().await,
            StorageService::Gridfs(gridfs) => gridfs.init().await,
            StorageService::S3(s3) => s3.init().await,
        }
    }

    async fn open(&self, path: impl AsRef<Path> + Send) -> Result<Option<Bytes>> {
        match self {
            StorageService::Filesystem(fs) => fs.open(path).await,
            StorageService::Gridfs(gridfs) => gridfs.open(path).await,
            StorageService::S3(s3) => s3.open(path).await,
        }
    }

    async fn blob(&self, path: impl AsRef<Path> + Send) -> Result<Option<Blob>> {
        match self {
            StorageService::Filesystem(fs) => fs.blob(path).await,
            StorageService::Gridfs(gridfs) => gridfs.blob(path).await,
            StorageService::S3(s3) => s3.blob(path).await,
        }
    }

    async fn blobs(
        &self,
        path: Option<impl AsRef<Path> + Send>,
        options: Option<ListBlobsRequest>,
    ) -> Result<Vec<Blob>> {
        match self {
            StorageService::Filesystem(fs) => fs.blobs(path, options).await,
            StorageService::Gridfs(gridfs) => gridfs.blobs(path, options).await,
            StorageService::S3(s3) => s3.blobs(path, options).await,
        }
    }

    async fn delete(&self, path: impl AsRef<Path> + Send) -> Result<()> {
        match self {
            StorageService::Filesystem(fs) => fs.delete(path).await,
            StorageService::Gridfs(gridfs) => gridfs.delete(path).await,
            StorageService::S3(s3) => s3.delete(path).await,
        }
    }

    async fn exists(&self, path: impl AsRef<Path> + Send) -> Result<bool> {
        match self {
            StorageService::Filesystem(fs) => fs.exists(path).await,
            StorageService::Gridfs(gridfs) => gridfs.exists(path).await,
            StorageService::S3(s3) => s3.exists(path).await,
        }
    }

    async fn upload(&self, path: impl AsRef<Path> + Send, options: UploadRequest) -> Result<()> {
        match self {
            StorageService::Filesystem(fs) => fs.upload(path, options).await,
            StorageService::Gridfs(gridfs) => gridfs.upload(path, options).await,
            StorageService::S3(s3) => s3.upload(path, options).await,
        }
    }
}
