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
use azalia::TRUTHY_REGEX;
use azure_storage::CloudLocation;
use bson::Document;
use eyre::{Context, Report};
use mongodb::options::{
    Acknowledgment, AuthMechanism, ClientOptions, ReadConcern, ReadPreference,
    ReadPreferenceOptions, SelectionCriteria, ServerAddress, TagSet, WriteConcern,
};
use noelware_config::{
    env,
    merge::{strategy, Merge},
    TryFromEnv,
};
use remi_azure::Credential;
use serde::{Deserialize, Serialize};
use std::{borrow::Cow, env::VarError, path::PathBuf, str::FromStr};

/// Represents the configuration for configuring
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Config {
    GridFS(remi_gridfs::StorageConfig),
    Azure(remi_azure::StorageConfig),
    Filesystem(remi_fs::Config),
    S3(remi_s3::StorageConfig),
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

                "s3" => Ok(Config::S3(remi_s3::StorageConfig {
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

                "gridfs" => Ok(Config::GridFS(remi_gridfs::StorageConfig {
                    selection_criteria: match env!("UME_STORAGE_GRIDFS_SELECTION_CRITERIA") {
                        Ok(value) => match value.as_str() {
                            "primary" => Some(SelectionCriteria::ReadPreference(ReadPreference::Primary)),
                            "secondary" => Some(SelectionCriteria::ReadPreference(ReadPreference::Secondary { options: parse_read_preference_options()? })),
                            "primary-preferred" => Some(SelectionCriteria::ReadPreference(ReadPreference::PrimaryPreferred { options: parse_read_preference_options()? })),
                            "secondary-preferred" => Some(SelectionCriteria::ReadPreference(ReadPreference::SecondaryPreferred { options: parse_read_preference_options()? })),
                            "nearest" => Some(SelectionCriteria::ReadPreference(ReadPreference::Nearest { options: parse_read_preference_options()? })),
                            val => return Err(eyre!("unknown value [{val}] for env `$UME_STORAGE_GRIDFS_SELECTION_CRITERIA`, expected one of: [primary, secondary, primary-preferred, secondary-preferred, nearest]"))
                        },

                        Err(std::env::VarError::NotPresent) => None,
                        Err(_) => return Err(eyre!("received invalid UTF-8 for environment variable `$UME_STORAGE_GRIDFS_SELECTION_CRITERIA`")),
                    },

                    chunk_size: match env!("UME_STORAGE_GRIDFS_CHUNK_SIZE") {
                        Ok(value) => value.parse().map(Some).with_context(|| format!("unable to parse `{value}` as a u32"))?,
                        Err(std::env::VarError::NotPresent) => None,
                        Err(_) => return Err(eyre!("received invalid UTF-8 for environment variable `$UME_STORAGE_GRIDFS_CHUNK_SIZE`")),
                    },

                    write_concern: match env!("UME_STORAGE_GRIDFS_WRITE_CONCERN") {
                        Ok(val) => Some(WriteConcern::builder()
                            .journal(env!("UME_STORAGE_GRIDFS_WRITE_CONCERN_JOURNAL", |val| TRUTHY_REGEX.is_match(&val); or false))
                            .w(match val.parse::<u32>() {
                                Ok(val) => Acknowledgment::Nodes(val),
                                Err(_) => match val.as_str() {
                                    "majority" => Acknowledgment::Majority,
                                    s => Acknowledgment::Custom(s.to_owned())
                                }
                            })
                            .w_timeout(match env!("UME_STORAGE_GRIDFS_WRITE_CONCERN_TIMEOUT") {
                                Ok(val) => Some(humantime::parse_duration(&val)?),
                                Err(std::env::VarError::NotPresent) => None,
                                Err(_) => return Err(eyre!("received invalid UTF-8 for environment variable `$UME_STORAGE_GRIDFS_WRITE_CONCERN_TIMEOUT`")),
                            })
                            .build()),

                        Err(std::env::VarError::NotPresent) => None,
                        Err(_) => return Err(eyre!("received invalid UTF-8 for environment variable `$UME_STORAGE_GRIDFS_WRITE_CONCERN`")),
                    },

                    read_concern: match env!("UME_STORAGE_GRIDFS_READ_CONCERN") {
                        Ok(val) => Some(match val.as_str() {
                            "majority" => ReadConcern::majority(),
                            "linear" | "linearizable" => ReadConcern::linearizable(),
                            "local" => ReadConcern::local(),
                            "avaliable" => ReadConcern::available(),
                            "snapshot" => ReadConcern::snapshot(),
                            s => ReadConcern::custom(s.to_owned())
                        }),

                        Err(std::env::VarError::NotPresent) => None,
                        Err(_) => return Err(eyre!("received invalid UTF-8 for environment variable `$UME_STORAGE_GRIDFS_READ_CONCERN`")),
                    },

                    database: env!("UME_STORAGE_GRIDFS_DATABASE", optional),
                    client_options: parse_mongo_client_options()?,
                    bucket: env!("UME_STORAGE_GRIDFS_BUCKET", optional)
                        .unwrap_or("ume".into())
                })),

                loc => Err(eyre!(
                    "expected [filesystem/fs, azure, s3, gridfs]; received '{loc}'"
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

fn merge_s3(config: &mut remi_s3::StorageConfig, right: remi_s3::StorageConfig) {
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

fn parse_read_preference_options() -> eyre::Result<ReadPreferenceOptions> {
    Ok(ReadPreferenceOptions::builder()
        .tag_sets(match env!("UME_STORAGE_GRIDFS_READ_PREFERENCE_TAG_SETS") {
            Ok(value) => {
                let mut values = Vec::new();
                for line in value.split(',') {
                    if let Some((key, value)) = line.split_once('=') {
                        if value.contains('=') {
                            continue;
                        }

                        values.push(TagSet::from_iter([(key.into(), value.into())]));
                    }
                }

                Some(values)
            }

            Err(std::env::VarError::NotPresent) => None,
            Err(_) => return Err(eyre!("")),
        })
        .max_staleness(
            match env!("UME_STORAGE_GRIDFS_READ_PREFERENCE_MAX_STALENESS") {
                Ok(value) => Some(humantime::parse_duration(&value)?),
                Err(std::env::VarError::NotPresent) => None,
                Err(_) => return Err(eyre!("unable to provide `$UME_STORAGE_GRIDFS_READ_PREFERENCE_MAX_STALENESS` as a valid UTF-8 string")),
            },
        )
        .build())
}

fn parse_mongo_client_options() -> eyre::Result<ClientOptions> {
    Ok(ClientOptions::builder()
        .app_name(env!("UME_STORAGE_GRIDFS_APP_NAME", optional))
        .connect_timeout(match env!("UME_STORAGE_GRIDFS_CONNECT_TIMEOUT") {
            Ok(value) => Some(humantime::parse_duration(&value)?),
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => {
                return Err(eyre!(
                    "unable to provide `$UME_STORAGE_GRIDFS_CONNECT_TIMEOUT` as a valid UTF-8 string"
                ))
            }
        })
        .credential(match env!("UME_STORAGE_GRIDFS_CREDENTIALS") {
            Ok(value) => {
                let (user, pass) = value
                    .split_once(':')
                    .ok_or_else(|| eyre!("must provide the template 'username:password'"))
                    .context("if there is no username, but a password, do: \":<password>\"")
                    .context("if there is no password, but a username, do: \"<username>:\"")?;

                Some(
                    mongodb::options::Credential::builder()
                        .source(env!("UME_STORAGE_GRIDFS_CREDENTIAL_SOURCE", optional))
                        .username(if user.is_empty() {
                            None
                        } else {
                            Some(user.to_owned())
                        })
                        .password(if pass.is_empty() {
                            None
                        } else {
                            Some(pass.to_owned())
                        })
                        .mechanism(match env!("UME_STORAGE_GRIDFS_CREDENTIAL_MECHANISM") {
                            Ok(value) => match value.as_str() {
                                "SCRAM-SHA1" => Some(AuthMechanism::ScramSha1),
                                "SCRAM-SHA256" => Some(AuthMechanism::ScramSha256),
                                "X509" => Some(AuthMechanism::MongoDbX509),
                                "GSSAPI" | "Gssapi" => Some(AuthMechanism::Gssapi),
                                "PLAIN" => Some(AuthMechanism::Plain),
                                _ => None
                            },
                            Err(std::env::VarError::NotPresent) => None,
                            Err(_) => return Err(eyre!("unable to provide `$UME_STORAGE_GRIDFS_CREDENTIAL_MECHANISM` as a valid UTF-8 string"))
                        })
                        .mechanism_properties(match env!("UME_STORAGE_GRIDFS_CREDENTIAL_MECHANISM_PROPERTIES") {
                            Ok(value) => {
                                let mut map = Document::new();
                                for item in value.split(',') {
                                    if let Some((key, value)) = item.split_once('=') {
                                        if value.contains('=') { continue; }

                                        map.insert(key, value.to_owned());
                                    }
                                }

                                match value.is_empty() {
                                    true => None,
                                    false => Some(map)
                                }
                            },
                            Err(std::env::VarError::NotPresent) => None,
                            Err(_) => return Err(eyre!("unable to provide `$UME_STORAGE_GRIDFS_CREDENTIAL_MECHANISM_PROPERTIES` as a valid UTF-8 string"))
                        })
                        .build(),
                )
            }
            Err(std::env::VarError::NotPresent) => None,
            Err(_) => {
                return Err(eyre!(
                    "unable to provide `$UME_STORAGE_GRIDFS_CREDENTIALS` as a valid UTF-8 string"
                ))
            }
        })
        .hosts(match env!("UME_STORAGE_GRIDFS_SERVERS") {
            Ok(value) => value
                .split(',')
                .map(|x| ServerAddress::from_str(x).map_err(Report::from))
                .collect::<eyre::Result<Vec<_>>>()?,

            Err(std::env::VarError::NotPresent) => {
                vec![ServerAddress::from_str("mongo://localhost:27017")?]
            }

            Err(_) => {
                return Err(eyre!(
                    "unable to provide `$UME_STORAGE_GRIDFS_SERVERS` as a valid UTF-8 string"
                ))
            }
        })
        .build())
}
