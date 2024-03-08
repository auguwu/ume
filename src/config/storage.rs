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

use aws_sdk_s3::{
    config::Region,
    types::{BucketCannedAcl, ObjectCannedAcl},
};
use azure_storage::CloudLocation;
use eyre::{Context, Report};
use noelware_config::{
    env,
    merge::{strategy, Merge},
    TryFromEnv,
};
use remi_azure::Credential;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, env::VarError, path::PathBuf, str::FromStr};

use crate::TRUTHY_REGEX;

/// Represents the configuration for configuring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Config {
    GridFS(remi_gridfs::StorageConfig),
    Azure(remi_azure::StorageConfig),
    Filesystem(remi_fs::Config),
    S3(remi_s3::S3StorageConfig),
}

impl Default for Config {
    fn default() -> Config {
        Config::Filesystem(remi_fs::Config {
            directory: PathBuf::from("./data"),
        })
    }
}

impl TryFromEnv for Config {
    type Output = Config;
    type Error = Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        match env!("UME_STORAGE_SERVICE") {
            Ok(res) => match res.to_lowercase().as_str() {
                "filesystem" | "fs" => Ok(Config::Filesystem(remi_fs::Config {
                    directory: env!("UME_STORAGE_FILESYSTEM_DIRECTORY", |val| val.parse::<PathBuf>().unwrap()).unwrap_or_else(|_| PathBuf::from("./data")),
                })),

                "azure" => Ok(Config::Azure(remi_azure::StorageConfig {
                    credentials: to_env_credentials()?,
                    location: to_env_location()?,
                    container: env!("UME_STORAGE_AZURE_CONTAINER", optional)
                        .unwrap_or("ume".into()),
                })),

                "s3" => Ok(Config::S3(remi_s3::S3StorageConfig {
                    enable_signer_v4_requests: env!("UME_STORAGE_S3_ENABLE_SIGNER_V4_REQUESTS", |val| TRUTHY_REGEX.is_match(&val); or false),
                    enforce_path_access_style: env!("UME_STORAGE_S3_ENFORCE_PATH_ACCESS_STYLE", |val| TRUTHY_REGEX.is_match(&val); or false),
                    default_object_acl: env!("UME_STORAGE_S3_DEFAULT_OBJECT_ACL", |val| ObjectCannedAcl::from_str(val.as_str()).ok(); or Some(ObjectCannedAcl::BucketOwnerFullControl)),
                    default_bucket_acl: env!("UME_STORAGE_S3_DEFAULT_OBJECT_ACL", |val| BucketCannedAcl::from_str(val.as_str()).ok(); or Some(BucketCannedAcl::AuthenticatedRead)),

                    secret_access_key: env!("UME_STORAGE_S3_SECRET_ACCESS_KEY").map_err(|e| match e {
                        VarError::NotPresent => eyre!("you're required to add the [UME_STORAGE_S3_SECRET_ACCESS_KEY] environment variable"),
                        VarError::NotUnicode(_) => eyre!("wanted valid UTF-8 for env `UME_STORAGE_S3_SECRET_ACCESS_KEY`")
                    })?,

                    access_key_id: env!("UME_STORAGE_S3_ACCESS_KEY_ID").map_err(|e| match e {
                        VarError::NotPresent => eyre!("you're required to add the [UME_STORAGE_S3_ACCESS_KEY_ID] environment variable"),
                        VarError::NotUnicode(_) => eyre!("wanted valid UTF-8 for env `UME_STORAGE_S3_ACCESS_KEY_ID`")
                    })?,

                    app_name: env!("UME_STORAGE_S3_APP_NAME", optional),
                    endpoint: env!("UME_STORAGE_S3_ENDPOINT", optional),
                    prefix: env!("UME_STORAGE_S3_PREFIX", optional),
                    region: env!("UME_STORAGE_S3_REGION", |val| Some(Region::new(Cow::Owned(val))); or Some(Region::new(Cow::Borrowed("us-east-1")))),
                    bucket: env!("UME_STORAGE_S3_BUCKET", optional)
                        .unwrap_or("ume".into()),
                })),

                loc => Err(eyre!(
                    "expected [filesystem/fs, azure, s3]; received '{loc}'"
                )),
            },

            Err(std::env::VarError::NotPresent) => Ok(Default::default()),
            Err(e) => Err(Report::from(e)),
        }
    }
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        match (self.clone(), other) {
            (Config::GridFS(mut gridfs1), Config::GridFS(gridfs2)) => {
                merge_gridfs(&mut gridfs1, gridfs2);
                *self = Config::GridFS(gridfs1);
            }

            (Config::Azure(mut azure1), Config::Azure(azure2)) => {
                merge_azure(&mut azure1, azure2);
                *self = Config::Azure(azure1);
            }

            (Config::S3(mut s3_1), Config::S3(s3_2)) => {
                merge_s3(&mut s3_1, s3_2);
                *self = Config::S3(s3_1);
            }

            (Config::Filesystem(mut fs1), Config::Filesystem(fs2)) => {
                merge_fs(&mut fs1, fs2);
                *self = Config::Filesystem(fs1);
            }

            (_, other) => {
                *self = other;
            }
        }
    }
}

macro_rules! merge_tuple {
    ($first:expr, $second:expr, copyable) => {
        match ($first, $second) {
            (Some(obj1), Some(obj2)) if obj1 != obj2 => {
                $first = Some(obj2);
            }

            (None, Some(obj)) => {
                $first = Some(obj);
            }

            _ => {}
        }
    };

    ($first:expr, $second:expr) => {
        match (&($first), &($second)) {
            (Some(obj1), Some(obj2)) if obj1 != obj2 => {
                $first = Some(obj2.clone());
            }

            (None, Some(obj)) => {
                $first = Some(obj.clone());
            }

            _ => {}
        }
    };
}

fn merge_gridfs(config: &mut remi_gridfs::StorageConfig, right: remi_gridfs::StorageConfig) {
    merge_tuple!(config.selection_criteria, right.selection_criteria);
    merge_tuple!(config.write_concern, right.write_concern);
    merge_tuple!(config.read_concern, right.read_concern);
    merge_tuple!(config.chunk_size, right.chunk_size, copyable);
    config.bucket.merge(right.bucket);
}

fn merge_azure(config: &mut remi_azure::StorageConfig, right: remi_azure::StorageConfig) {
    match (&config.location, &right.location) {
        (CloudLocation::Public { account: acc1 }, CloudLocation::Public { account: acc2 })
            if acc1 != acc2 =>
        {
            config.location = CloudLocation::Public {
                account: acc2.clone(),
            };
        }

        (CloudLocation::China { account: acc1 }, CloudLocation::China { account: acc2 })
            if acc1 != acc2 =>
        {
            config.location = CloudLocation::China {
                account: acc2.clone(),
            };
        }

        (
            CloudLocation::Emulator {
                address: addr1,
                port: port1,
            },
            CloudLocation::Emulator {
                address: addr2,
                port: port2,
            },
        ) if addr1 != addr2 || port1 != port2 => {
            config.location = CloudLocation::Emulator {
                address: addr2.clone(),
                port: *port2,
            };
        }

        (_, other) => {
            config.location = other.clone();
        }
    };

    match (&config.credentials, &right.credentials) {
        (
            Credential::AccessKey {
                account: acc1,
                access_key: ak1,
            },
            Credential::AccessKey {
                account,
                access_key,
            },
        ) if acc1 != account || access_key != ak1 => {
            config.credentials = Credential::AccessKey {
                account: account.clone(),
                access_key: access_key.clone(),
            };
        }

        (Credential::SASToken(token1), Credential::SASToken(token2)) if token1 != token2 => {
            config.credentials = Credential::SASToken(token2.to_owned());
        }

        (Credential::Bearer(token1), Credential::Bearer(token2)) if token1 != token2 => {
            config.credentials = Credential::SASToken(token2.to_owned());
        }

        (Credential::Anonymous, Credential::Anonymous) => {}

        // overwrite if they aren't the same at all
        (_, other) => {
            config.credentials = other.clone();
        }
    };

    config.container.merge(right.container);
}

fn merge_fs(config: &mut remi_fs::Config, right: remi_fs::Config) {
    if config.directory != right.directory {
        config.directory = right.directory;
    }
}

fn merge_s3(config: &mut remi_s3::S3StorageConfig, right: remi_s3::S3StorageConfig) {
    strategy::bool::only_if_falsy(
        &mut config.enable_signer_v4_requests,
        right.enable_signer_v4_requests,
    );

    strategy::bool::only_if_falsy(
        &mut config.enforce_path_access_style,
        right.enforce_path_access_style,
    );

    merge_tuple!(config.default_bucket_acl, right.default_bucket_acl);
    merge_tuple!(config.default_object_acl, right.default_object_acl);

    config.secret_access_key.merge(right.secret_access_key);
    config.access_key_id.merge(right.access_key_id);

    merge_tuple!(config.app_name, right.app_name);
    merge_tuple!(config.endpoint, right.endpoint);
    merge_tuple!(config.region, right.region);

    config.bucket.merge(right.bucket);
}

fn to_env_credentials() -> eyre::Result<Credential> {
    match env!("UME_STORAGE_AZURE_CREDENTIAL") {
        Ok(res) => match res.as_str() {
            "anonymous" | "anon" => Ok(Credential::Anonymous),
            "accesskey" | "access_key" => Ok(Credential::AccessKey {
                account: env!("UME_STORAGE_AZURE_CREDENTIAL_ACCESSKEY_ACCOUNT")
                    .context("missing required env variable [UME_STORAGE_AZURE_CREDENTIAL_ACCESSKEY_ACCOUNT]")?,
                access_key: env!("UME_STORAGE_AZURE_CREDENTIAL_ACCESSKEY")
                    .context("missing required env variable [UME_STORAGE_AZURE_CREDENTIAL_ACCESSKEY]")?,
            }),

            "sastoken" | "sas_token" => Ok(Credential::SASToken(
                env!("UME_STORAGE_AZURE_CREDENTIAL_SAS_TOKEN")
                    .context("missing required env variable [UME_STORAGE_AZURE_CREDENTIAL_SAS_TOKEN]")?,
            )),

            "bearer" => Ok(Credential::SASToken(
                env!("UME_STORAGE_AZURE_CREDENTIAL_BEARER")
                    .context("missing required env variable [UME_STORAGE_AZURE_CREDENTIAL_BEARER]")?,
            )),

            res => Err(eyre!(
                "expected [anonymous/anon, accesskey/access_key, sastoken/sas_token, bearer]; received '{res}'"
            )),
        },
        Err(_) => Err(eyre!(
            "missing required `UME_STORAGE_AZURE_CREDENTIAL` env or was invalid utf-8"
        )),
    }
}

fn to_env_location() -> eyre::Result<azure_storage::CloudLocation> {
    match env!("UME_STORAGE_AZURE_LOCATION") {
        Ok(res) => match res.as_str() {
            "public" => Ok(azure_storage::CloudLocation::Public {
                account: env!("UME_STORAGE_AZURE_ACCOUNT")
                    .context("missing required env [UME_STORAGE_AZURE_ACCOUNT]")?,
            }),

            "china" => Ok(azure_storage::CloudLocation::China {
                account: env!("UME_STORAGE_AZURE_ACCOUNT")
                    .context("missing required env [UME_STORAGE_AZURE_ACCOUNT]")?,
            }),

            "emulator" => Ok(azure_storage::CloudLocation::Emulator {
                address: env!("UME_STORAGE_AZURE_EMULATOR_ADDRESS")
                    .context("missing required env [UME_STORAGE_AZURE_EMULATOR_ADDRESS]")?,

                port: match env!("UME_STORAGE_AZURE_EMULATOR_PORT") {
                    Ok(res) => res.parse::<u16>()?,
                    Err(VarError::NotPresent) => {
                        return Err(eyre!(
                            "missing `UME_STORAGE_AZURE_EMULATOR_PORT` environment variable"
                        ))
                    }

                    Err(VarError::NotUnicode(_)) => {
                        return Err(eyre!(
                            "`UME_STORAGE_AZURE_EMULATOR_PORT` env was not in valid UTF-8"
                        ))
                    }
                },
            }),

            "custom" => Ok(azure_storage::CloudLocation::Custom {
                account: env!("UME_STORAGE_AZURE_ACCOUNT")
                    .context("missing required env [UME_STORAGE_AZURE_ACCOUNT]")?,

                uri: env!("UME_STORAGE_AZURE_URI")
                    .context("missing required env [UME_STORAGE_AZURE_URI]")?,
            }),

            loc => Err(eyre!(
                "expected [public, china, emulator, custom]; received '{loc}'"
            )),
        },

        Err(_) => Err(eyre!(
            "missing required `UME_STORAGE_AZURE_LOCATION` env or was invalid utf-8"
        )),
    }
}
