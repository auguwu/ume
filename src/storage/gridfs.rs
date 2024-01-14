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

use async_trait::async_trait;
use bytes::{Bytes, BytesMut};
use futures_util::StreamExt;
use mongodb::{
    bson::{doc, raw::ValueAccessErrorKind, Bson, RawDocument},
    gridfs::GridFsBucket,
    options::GridFsFindOptions,
    Database,
};
use remi_core::{Blob, FileBlob, ListBlobsRequest, StorageService, UploadRequest};
use std::{io, path::Path, sync::Arc};
use tokio_util::{compat::FuturesAsyncReadCompatExt, io::ReaderStream};
use tracing::{error, info, warn};

fn to_io_error(error: mongodb::error::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, format!("mongodb error: {}", error))
}

fn value_access_err_to_io_error(error: mongodb::bson::raw::ValueAccessError) -> io::Error {
    match error.kind {
        ValueAccessErrorKind::NotPresent => io::Error::new(
            io::ErrorKind::NotFound,
            format!("key [{}] was not found", error.key()),
        ),

        ValueAccessErrorKind::UnexpectedType {
            expected, actual, ..
        } => io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "expected type '{expected:?}', but the actual type for key [{}] is '{actual:?}'",
                error.key()
            ),
        ),

        ValueAccessErrorKind::InvalidBson(err) => {
            io::Error::new(io::ErrorKind::Other, format!("bson error: {err}"))
        }

        _ => unreachable!(),
    }
}

fn blob(bytes: Bytes, doc: &RawDocument) -> Result<FileBlob, io::Error> {
    let filename = doc
        .get_str("filename")
        .map_err(value_access_err_to_io_error)?;

    let length = doc
        .get_i64("length")
        .map_err(value_access_err_to_io_error)?;

    let content_type = doc
        .get_str("contentType")
        .map_err(value_access_err_to_io_error)?;

    let created_at = doc
        .get_datetime("uploadDate")
        .map_err(value_access_err_to_io_error)?;

    Ok(FileBlob::new(
        None,
        Some(content_type.to_owned()),
        if created_at.timestamp_millis() < 0 {
            warn!(remi.service = "gridfs", %filename, "created_at timestamp was negative!");
            None
        } else {
            Some(
                u128::try_from(created_at.timestamp_millis())
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            )
        },
        false,
        String::from("gridfs"),
        bytes,
        filename.to_owned(),
        if length < 0 {
            0
        } else {
            length
                .try_into()
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        },
    ))
}

#[derive(Debug, Clone)]
pub struct GridfsStorageService {
    bucket: Arc<GridFsBucket>,
}

impl GridfsStorageService {
    /// Creates a new [`GridfsStorageService`] with the MongoDB database and configuration options to configure this.
    /// It calls the [`GridfsStorageService::with_bucket`] function internally to get a instance of this service.
    pub fn new(db: &Database, config: GridfsStorageConfig) -> GridfsStorageService {
        let bucket = db.gridfs_bucket(Some(config.to_gridfs_options()));
        GridfsStorageService {
            bucket: Arc::new(bucket),
        }
    }
}

#[async_trait]
impl StorageService for GridfsStorageService {
    fn name(self) -> &'static str {
        "ume:gridfs"
    }

    async fn open(&self, path: impl AsRef<Path> + Send) -> io::Result<Option<Bytes>> {
        let path = path.as_ref();
        info!(remi.service = "gridfs", "opening file [{}]", path.display());

        // ensure that the `path` is utf-8 encoded, because I think
        // MongoDB expects strings to be utf-8 encoded?
        let path_str = path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "expected utf-8 encoded path string",
            )
        })?;

        let mut cursor = self
            .bucket
            .find(
                doc! {
                    "filename": path_str,
                },
                GridFsFindOptions::default(),
            )
            .await
            .map_err(to_io_error)?;

        // has_advanced returns false if there is no entries that have that filename
        let has_advanced = cursor.advance().await.map_err(to_io_error)?;
        if !has_advanced {
            warn!("file [{}] doesn't exist", path.display());
            return Ok(None);
        }

        let doc = cursor.current();
        let stream = self
            .bucket
            .open_download_stream(Bson::ObjectId(
                doc.get_object_id("_id")
                    .map_err(value_access_err_to_io_error)?,
            ))
            .await
            .map_err(to_io_error)?;

        let mut bytes = BytesMut::new();
        let mut reader = ReaderStream::new(stream.compat());
        while let Some(raw) = reader.next().await {
            match raw {
                Ok(b) => bytes.extend(b),
                Err(e) => return Err(e),
            }
        }

        Ok(Some(bytes.into()))
    }

    async fn blob(&self, path: impl AsRef<Path> + Send) -> io::Result<Option<Blob>> {
        let path = path.as_ref();
        let Some(bytes) = self.open(path).await? else {
            return Ok(None);
        };

        // .unwrap() is safe here since .open() validates if the path is a
        // utf-8 string.
        let path_str = path.to_str().unwrap();

        info!(
            remi.service = "gridfs",
            "getting file metadata for file [{}]",
            path.display(),
        );

        let mut cursor = self
            .bucket
            .find(
                doc! {
                    "filename": path_str,
                },
                GridFsFindOptions::default(),
            )
            .await
            .map_err(to_io_error)?;

        // has_advanced returns false if there is no entries that have that filename
        let has_advanced = cursor.advance().await.map_err(to_io_error)?;
        if !has_advanced {
            warn!("file [{}] doesn't exist", path.display());
            return Ok(None);
        }

        let doc = cursor.current();
        let blob = blob(bytes, doc)?;

        Ok(Some(Blob::File(blob)))
    }

    async fn blobs(
        &self,
        path: Option<impl AsRef<Path> + Send>,
        _request: Option<ListBlobsRequest>,
    ) -> io::Result<Vec<Blob>> {
        // TODO(@auguwu): support filtering files, for now we should probably
        // heavily test this
        if let Some(path) = path {
            let path = path.as_ref();
            warn!(
                remi.service = "gridfs",
                "using blobs() with a path [{}] is not supported",
                path.display()
            );

            return Ok(vec![]);
        }

        let mut cursor = self
            .bucket
            .find(doc!(), GridFsFindOptions::default())
            .await
            .map_err(to_io_error)?;

        let mut blobs = vec![];
        while cursor.advance().await.map_err(to_io_error)? {
            let doc = cursor.current();
            let stream = self
                .bucket
                .open_download_stream(Bson::ObjectId(
                    doc.get_object_id("_id")
                        .map_err(value_access_err_to_io_error)?,
                ))
                .await
                .map_err(to_io_error)?;

            let mut bytes = BytesMut::new();
            let mut reader = ReaderStream::new(stream.compat());
            while let Some(raw) = reader.next().await {
                match raw {
                    Ok(b) => bytes.extend(b),
                    Err(e) => return Err(e),
                }
            }

            match blob(bytes.into(), doc) {
                Ok(blob) => blobs.push(Blob::File(blob)),
                Err(e) => {
                    error!(remi.service = "gridfs", error = %e, "unable to convert to FileBlob");
                }
            }
        }

        Ok(blobs)
    }

    async fn delete(&self, path: impl AsRef<Path> + Send) -> io::Result<()> {
        let path = path.as_ref();
        info!(
            remi.service = "gridfs",
            "deleting file [{}]",
            path.display()
        );

        // ensure that the `path` is utf-8 encoded, because I think
        // MongoDB expects strings to be utf-8 encoded?
        let path_str = path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "expected utf-8 encoded path string",
            )
        })?;

        let mut cursor = self
            .bucket
            .find(
                doc! {
                    "filename": path_str,
                },
                GridFsFindOptions::default(),
            )
            .await
            .map_err(to_io_error)?;

        // has_advanced returns false if there is no entries that have that filename
        let has_advanced = cursor.advance().await.map_err(to_io_error)?;
        if !has_advanced {
            warn!("file [{}] doesn't exist", path.display());
            return Ok(());
        }

        let doc = cursor.current();
        let oid = doc
            .get_object_id("_id")
            .map_err(value_access_err_to_io_error)?;

        self.bucket
            .delete(Bson::ObjectId(oid))
            .await
            .map_err(to_io_error)
    }

    async fn exists(&self, path: impl AsRef<Path> + Send) -> io::Result<bool> {
        match self.open(path).await {
            Ok(Some(_)) => Ok(true),
            Ok(None) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn upload(
        &self,
        _path: impl AsRef<Path> + Send,
        _options: UploadRequest,
    ) -> io::Result<()> {
        Ok(())
    }
}
