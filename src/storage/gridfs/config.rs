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

use crate::{config::FromEnv, var};
use mongodb::options::{GridFsBucketOptions, ReadConcern, WriteConcern};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GridfsStorageConfig {
    /// Allows Ume to specify the level of acknowledgement requested from the MongoDB server
    /// for write operations, read the [MongoDB documentation](https://www.mongodb.com/docs/manual/reference/write-concern/) for more information.
    ///
    /// ## Example
    /// `config/ume.yaml`:
    ///
    /// ```yaml
    /// storage:
    ///     gridfs:
    ///         write_concern:
    ///             w: 12 # 12 nodes
    ///             w_timeout: 1000 # write timeout (1s)
    ///             journal: true # uses on-disk journal
    /// ```
    ///
    /// * Required: No
    /// * Configuration Key: `config.storage.gridfs.write_concern` (object)
    ///
    /// ## Environment Variables
    ///
    /// | Name                                       | Type                                    | Description                                                                                                                          | Required |
    /// | :----------------------------------------- | :-------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------- | :------- |
    /// | `UME_STORAGE_GRIDFS_WRITE_CONCERN`         | `"majority"` or nothing for flexibility | `"majority"` can be only set to apply the majority options for a [`WriteConcern`].                                                   | No.      |
    /// | `UME_STORAGE_GRIDFS_WRITE_CONCERN_ACK`     | int (nodes), string (custom)            | configures the write acknowledgement if the write concern wasn't `"majority"`.                                                       | No.      |
    /// | `UME_STORAGE_GRIDFS_WRITE_CONCERN_TIMEOUT` | duration (string/int in millis)         | the timeout for the configured write concern if not the majority write concern was specified.                                        | No.      |
    /// | `UME_STORAGE_GRIDFS_WRITE_CONCERN_JOURNAL` | boolean                                 | if the write concern should configure all MongoDB request acknowledgments on operations that were propagated on the on-disk journal. | No.      |
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub write_concern: Option<WriteConcern>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_concern: Option<ReadConcern>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chunk_size: Option<u32>,

    pub bucket_name: String,
}

impl FromEnv for GridfsStorageConfig {
    type Output = GridfsStorageConfig;

    fn from_env() -> Self::Output {
        GridfsStorageConfig {
            write_concern: var!("UME_STORAGE_GRIDFS_WRITE_CONCERN", {
                or_else: None;
                mapper: |val| match val.as_str() {
                    "majority" => Some(WriteConcern::MAJORITY),
                    _ => Some(WriteConcern::default()),
                };
            }),

            read_concern: var!("UME_STORAGE_GRIDFS_READ_CONCERN", {
                or_else: None;
                mapper: |val| match val.as_str() {
                    "linearizable" => Some(ReadConcern::linearizable()),
                    "available" => Some(ReadConcern::available()),
                    "snapshot" => Some(ReadConcern::snapshot()),
                    "majority" => Some(ReadConcern::majority()),
                    "local" => Some(ReadConcern::local()),
                    _ => None,
                };
            }),
            ..Default::default()
        }
    }
}

impl GridfsStorageConfig {
    /// Turns this [`GridfsStorageConfig`] object into a [`GridFsBucketOptions`] object.
    pub fn to_gridfs_options(&self) -> GridFsBucketOptions {
        GridFsBucketOptions::builder()
            .bucket_name(self.bucket_name.clone())
            .chunk_size_bytes(self.chunk_size)
            .read_concern(self.read_concern.clone())
            .write_concern(self.write_concern.clone())
            .build()
    }
}
