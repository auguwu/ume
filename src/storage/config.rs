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

use super::gridfs::{self, GridfsStorageConfig};
use crate::{config::FromEnv, var, TRUTHY_REGEX};
use aws_sdk_s3::{
    config::Region,
    types::{BucketCannedAcl, ObjectCannedAcl},
};
use remi_fs::FilesystemStorageConfig;
use remi_s3::S3StorageConfig;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageConfig {
    Filesystem(FilesystemStorageConfig),
    Gridfs(gridfs::GridfsStorageConfig),
    S3(S3StorageConfig),
}

impl Default for StorageConfig {
    fn default() -> StorageConfig {
        let datadir = var!("UME_STORAGE_FS_DIRECTORY", or_else: String::from("./data"));
        StorageConfig::Filesystem(FilesystemStorageConfig::new(datadir))
    }
}

impl FromEnv for StorageConfig {
    type Output = StorageConfig;

    fn from_env() -> Self::Output {
        match var!("UME_STORAGE_SERVICE") {
            Err(_) => StorageConfig::default(),
            Ok(value) => match value.as_str() {
                "filesystem" | "fs" => StorageConfig::default(),
                "gridfs" => StorageConfig::Gridfs(GridfsStorageConfig::from_env()),
                "s3" => {
                    let enable_signer_v4_requests = var!("UME_STORAGE_S3_ENABLE_SIGNER_V4_REQUESTS", {
                        or_else: false;
                        mapper: |p| TRUTHY_REGEX.is_match(p.as_str());
                    });

                    let enforce_path_access_style = var!("UME_STORAGE_S3_ENFORCE_PATH_ACCESS_STYLE", {
                        or_else: false;
                        mapper: |p| TRUTHY_REGEX.is_match(p.as_str());
                    });

                    let default_object_acl = var!("UME_STORAGE_S3_DEFAULT_OBJECT_ACL", {
                        or_else: ObjectCannedAcl::BucketOwnerFullControl;
                        mapper: |p| ObjectCannedAcl::from_str(p.as_str()).unwrap_or_else(|e| panic!("unable to parse [{p}] as ObjectCannedAcl: {e}"));
                    });

                    let default_bucket_acl = var!("UME_STORAGE_S3_DEFAULT_BUCKET_ACL", {
                        or_else: BucketCannedAcl::Private;
                        mapper: |p| BucketCannedAcl::from_str(p.as_str()).unwrap_or_else(|e| panic!("unable to parse [{p}] as BucketCannedAcl: {e}"));
                    });

                    let secret_access_key = var!("UME_STORAGE_S3_SECRET_ACCESS_KEY", or_else_do: |_| panic!("Missing required environment variable: `UME_STORAGE_S3_SECRET_ACCESS_KEY`."));
                    let access_key_id = var!("UME_STORAGE_S3_ACCESS_KEY_ID", or_else_do: |_| panic!("Missing required environment variable: `UME_STORAGE_S3_ACCESS_KEY_ID`."));
                    let region = var!("UME_STORAGE_S3_REGION", {
                        or_else: Region::new(Cow::Owned("us-east-1".to_owned()));
                        mapper: |val| Region::new(Cow::Owned(val));
                    });

                    let bucket = var!("UME_STORAGE_S3_BUCKET", or_else_do: |_| panic!("Missing required environment variable: `UME_STORAGE_S3_BUCKET`."));
                    let config = S3StorageConfig::default()
                        .with_enable_signer_v4_requests(enable_signer_v4_requests)
                        .with_enforce_path_access_style(enforce_path_access_style)
                        .with_default_bucket_acl(Some(default_bucket_acl))
                        .with_default_object_acl(Some(default_object_acl))
                        .with_secret_access_key(secret_access_key)
                        .with_access_key_id(access_key_id)
                        .with_region(Some(region))
                        .with_bucket(bucket)
                        .seal();

                    StorageConfig::S3(config)
                }

                _ => panic!("unknown storage service: [{value}]; expected 'filesystem', 'fs', 'gridfs', or 's3'")
            },
        }
    }
}
