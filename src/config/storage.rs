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

use azalia::{
    config::{env, merge::Merge, TryFromEnv},
    remi,
};
use serde::{Deserialize, Serialize};
use std::{env::VarError, path::PathBuf};

/// Represents the configuration for configuring the data storage where
/// ume will put all images in.
///
/// ## Examples
/// ### Filesystem
/// ```hcl
/// storage "filesystem" {
///   directory = "./some/directory/that/exists"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Config {
    /// Allows **ume** to connect to an external MongoDB server that allows
    /// GridFS features to be used.
    Gridfs(remi::gridfs::StorageConfig),

    /// Allows **ume** to use the local filesystem to store images in.
    Filesystem(remi::fs::StorageConfig),

    /// Allows **ume** to connect to Azure's Blob Storage to store images in.
    Azure(remi::azure::StorageConfig),

    /// Allows **ume** to connect to Amazon AWS S3 (or any compatible server) to store
    /// images in.
    S3(remi::s3::StorageConfig),
}

impl Default for Config {
    fn default() -> Config {
        Config::Filesystem(remi::fs::StorageConfig {
            directory: PathBuf::from("./data"),
        })
    }
}

impl TryFromEnv for Config {
    type Output = Config;
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self::Output, Self::Error> {
        match env!("UME_STORAGE_SERVICE") {
            Ok(input) => match &*input.to_ascii_lowercase() {
                "filesystem" | "fs" => Ok(Config::Filesystem(remi::fs::StorageConfig {
                    directory: env!("UME_STORAGE_FILESYSTEM_DIRECTORY", |val| PathBuf::from(val); or PathBuf::from("./data")),
                })),

                "gridfs" => Ok(Config::Gridfs(gridfs::create_config()?)),
                "azure" => Ok(Config::Azure(azure::create_config()?)),
                "s3" => Ok(Config::S3(s3::create_config()?)),
                input => Err(eyre!(
                    "unknown input [{input}]. expected either 'filesystem', 's3', 'azure', or 'gridfs'"
                )),
            },

            Err(VarError::NotPresent) => Ok(Default::default()),
            Err(VarError::NotUnicode(_)) => Err(eyre!(
                "environment variable `$UME_STORAGE_SERVICE` received invalid unicode as its value"
            )),
        }
    }
}

impl Merge for Config {
    fn merge(&mut self, other: Self) {
        match (self, other) {
            (Self::Filesystem(fs1), Self::Filesystem(fs2)) => {
                fs1.directory.merge(fs2.directory);
            }

            (Self::Gridfs(me), Self::Gridfs(other)) => {
                gridfs::merge_config(me, other);
            }

            (Self::Azure(me), Self::Azure(other)) => {
                azure::merge_config(me, other);
            }

            (Self::S3(me), Self::S3(other)) => {
                s3::merge_config(me, other);
            }

            (me, other) => {
                *me = other;
            }
        }
    }
}

pub(crate) mod s3 {
    use azalia::{
        config::{env, merge::Merge},
        remi::{
            self,
            s3::aws::{
                config::Region,
                s3::types::{BucketCannedAcl, ObjectCannedAcl},
            },
        },
        TRUTHY_REGEX,
    };
    use std::{borrow::Cow, env::VarError, str::FromStr};

    pub const ENABLE_SIGNER_V4_REQUESTS: &str = "UME_STORAGE_S3_ENABLE_SIGNER_V4_REQUESTS";
    pub const ENFORCE_PATH_ACCESS_STYLE: &str = "UME_STORAGE_S3_ENFORCE_PATH_ACCESS_STYLE";
    pub const DEFAULT_OBJECT_ACL: &str = "UME_STORAGE_S3_DEFAULT_OBJECT_ACL";
    pub const DEFAULT_BUCKET_ACL: &str = "UME_STORAGE_S3_DEFAULT_BUCKET_ACL";
    pub const SECRET_ACCESS_KEY: &str = "UME_STORAGE_S3_SECRET_ACCESS_KEY";
    pub const ACCESS_KEY_ID: &str = "UME_STORAGE_S3_ACCESS_KEY_ID";
    pub const APP_NAME: &str = "UME_STORAGE_S3_APP_NAME";
    pub const ENDPOINT: &str = "UME_STORAGE_S3_ENDPOINT";
    pub const PREFIX: &str = "UME_STORAGE_S3_PREFIX";
    pub const REGION: &str = "UME_STORAGE_S3_REGION";
    pub const BUCKET: &str = "UME_STORAGE_S3_BUCKET";

    const DEFAULT_OBJECT_CANNED_ACL: ObjectCannedAcl = ObjectCannedAcl::BucketOwnerFullControl;
    const DEFAULT_BUCKET_CANNED_ACL: BucketCannedAcl = BucketCannedAcl::AuthenticatedRead;

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

    pub fn create_config() -> eyre::Result<remi::s3::StorageConfig> {
        Ok(remi::s3::StorageConfig {
            enable_signer_v4_requests: env!(ENABLE_SIGNER_V4_REQUESTS, |val| TRUTHY_REGEX.is_match(&val); or false),
            enforce_path_access_style: env!(ENFORCE_PATH_ACCESS_STYLE, |val| TRUTHY_REGEX.is_match(&val); or false),
            default_object_acl: env!(DEFAULT_OBJECT_ACL, |val| ObjectCannedAcl::from_str(val.as_str()).ok(); or Some(DEFAULT_OBJECT_CANNED_ACL)),
            default_bucket_acl: env!(DEFAULT_BUCKET_ACL, |val| BucketCannedAcl::from_str(val.as_str()).ok(); or Some(DEFAULT_BUCKET_CANNED_ACL)),
            secret_access_key: env!(SECRET_ACCESS_KEY).map_err(|e| match e {
                VarError::NotPresent => {
                    eyre!("you're required to add the [{SECRET_ACCESS_KEY}] environment variable")
                }

                VarError::NotUnicode(_) => eyre!("wanted valid UTF-8 for env `{SECRET_ACCESS_KEY}`"),
            })?,

            access_key_id: env!(ACCESS_KEY_ID).map_err(|e| match e {
                VarError::NotPresent => {
                    eyre!("you're required to add the [{ACCESS_KEY_ID}] environment variable")
                }

                VarError::NotUnicode(_) => eyre!("wanted valid UTF-8 for env `{ACCESS_KEY_ID}`"),
            })?,

            app_name: env!(APP_NAME, optional),
            endpoint: env!(ENDPOINT, optional),
            prefix: env!(PREFIX, optional),
            region: env!(REGION, |val| Some(Region::new(Cow::Owned(val))); or Some(Region::new(Cow::Borrowed("us-east-1")))),
            bucket: env!(BUCKET, optional).unwrap_or("ume".into()),
        })
    }

    pub fn merge_config(me: &mut remi::s3::StorageConfig, other: remi::s3::StorageConfig) {
        azalia::config::merge::strategy::bool::only_if_falsy(
            &mut me.enable_signer_v4_requests,
            other.enable_signer_v4_requests,
        );

        azalia::config::merge::strategy::bool::only_if_falsy(
            &mut me.enforce_path_access_style,
            other.enforce_path_access_style,
        );

        merge_tuple!(me.default_bucket_acl, other.default_bucket_acl);
        merge_tuple!(me.default_object_acl, other.default_object_acl);

        me.secret_access_key.merge(other.secret_access_key);
        me.access_key_id.merge(other.access_key_id);

        merge_tuple!(me.app_name, other.app_name);
        merge_tuple!(me.endpoint, other.endpoint);
        merge_tuple!(me.region, other.region);

        me.bucket.merge(other.bucket);
    }
}

pub(crate) mod azure {
    use azalia::{
        config::{env, merge::Merge},
        remi::{
            self,
            azure::{CloudLocation, Credential},
        },
    };
    use eyre::Context;
    use std::env::VarError;

    pub const ACCESS_KEY_ACCOUNT: &str = "UME_STORAGE_AZURE_CREDENTIAL_ACCESSKEY_ACCOUNT";
    pub const ACCESS_KEY: &str = "UME_STORAGE_AZURE_CREDENTIAL_ACCESSKEY";
    pub const SAS_TOKEN: &str = "UME_STORAGE_AZURE_CREDENTIAL_SAS_TOKEN";
    pub const BEARER: &str = "UME_STORAGE_AZURE_CREDENTIAL_BEARER";

    pub const ACCOUNT: &str = "UME_STORAGE_AZURE_ACCOUNT";
    pub const URI: &str = "UME_STORAGE_AZURE_URI";

    pub const CREDENTIAL: &str = "UME_STORAGE_AZURE_CREDENTIAL";
    pub const CONTAINER: &str = "UME_STORAGE_AZURE_CONTAINER";
    pub const LOCATION: &str = "UME_STORAGE_AZURE_LOCATION";

    pub fn create_config() -> eyre::Result<remi::azure::StorageConfig> {
        Ok(remi::azure::StorageConfig {
            credentials: create_credentials_config()?,
            location: create_location()?,
            container: env!(CONTAINER, optional).unwrap_or("ume".into()),
        })
    }

    pub fn merge_config(me: &mut remi::azure::StorageConfig, other: remi::azure::StorageConfig) {
        me.container.merge(other.container);

        match (&me.location, &other.location) {
            (CloudLocation::Public(acc1), CloudLocation::Public(acc2)) if acc1 != acc2 => {
                me.location = CloudLocation::Public(acc2.clone());
            }

            (CloudLocation::China(acc1), CloudLocation::China(acc2)) if acc1 != acc2 => {
                me.location = CloudLocation::China(acc2.clone());
            }

            (_, other) => {
                me.location = other.clone();
            }
        }

        match (&me.credentials, &other.credentials) {
            (
                Credential::AccessKey {
                    account: acc1,
                    access_key: ak1,
                },
                Credential::AccessKey { account, access_key },
            ) if acc1 != account || access_key != ak1 => {
                me.credentials = Credential::AccessKey {
                    account: account.clone(),
                    access_key: access_key.clone(),
                };
            }

            (Credential::SASToken(token1), Credential::SASToken(token2)) if token1 != token2 => {
                me.credentials = Credential::SASToken(token2.to_owned());
            }

            (Credential::Bearer(token1), Credential::Bearer(token2)) if token1 != token2 => {
                me.credentials = Credential::SASToken(token2.to_owned());
            }

            (Credential::Anonymous, Credential::Anonymous) => {}

            // overwrite if they aren't the same at all
            (_, other) => {
                me.credentials = other.clone();
            }
        }
    }

    fn create_credentials_config() -> eyre::Result<remi::azure::Credential> {
        match env!(CREDENTIAL) {
            Ok(input) => match &*input.to_ascii_lowercase() {
                "anonymous" | "anon" | "" => Ok(remi::azure::Credential::Anonymous),
                "accesskey" | "access_key" | "access-key" => Ok(remi::azure::Credential::AccessKey {
                    account: env!(ACCESS_KEY_ACCOUNT).with_context(|| format!("missing required environment variable when `${CREDENTIAL}` is set to Access Key: `${ACCESS_KEY_ACCOUNT}`"))?,
                    access_key: env!(ACCESS_KEY).with_context(|| format!("missing required environment variable when `${CREDENTIAL}` is set to Access Key: `${ACCESS_KEY}`"))?
                }),

                "sastoken" | "sas-token" | "sas_token" => Ok(remi::azure::Credential::SASToken(
                    env!(SAS_TOKEN).with_context(|| format!("missing required environment variable when `${CREDENTIAL}` is set to SAS Token: `${SAS_TOKEN}`"))?
                )),

                "bearer" => Ok(remi::azure::Credential::Bearer(
                    env!(SAS_TOKEN).with_context(|| format!("missing required environment variable when `${CREDENTIAL}` is set to SAS Token: `${BEARER}`"))?
                )),

                input => Err(eyre!("unknown input [{input}] for `${CREDENTIAL}` environment variable"))
            },

            Err(VarError::NotPresent) => Ok(remi::azure::Credential::Anonymous),
            Err(VarError::NotUnicode(_)) => Err(eyre!("environment variable `${CREDENTIAL}` was invalid utf-8"))
        }
    }

    fn create_location() -> eyre::Result<CloudLocation> {
        match env!(LOCATION) {
            Ok(res) => match &*res.to_ascii_lowercase() {
                "public" | "" => {
                    Ok(CloudLocation::Public(env!(ACCOUNT).with_context(|| {
                        format!("missing required environment variable: [{ACCOUNT}]")
                    })?))
                }

                "china" => {
                    Ok(CloudLocation::China(env!(ACCOUNT).with_context(|| {
                        format!("missing required environment variable: [{ACCOUNT}]")
                    })?))
                }

                "custom" => Ok(CloudLocation::Custom {
                    account: env!(ACCOUNT)
                        .with_context(|| format!("missing required environment variable: [{ACCOUNT}]"))?,

                    uri: env!(URI).with_context(|| format!("missing required environment variable: [{ACCOUNT}]"))?,
                }),

                input => Err(eyre!(
                    "invalid option given: {input} | expected [public, china, custom]"
                )),
            },

            Err(VarError::NotPresent) => Err(eyre!("missing required environment variable: [{LOCATION}]")),
            Err(VarError::NotUnicode(_)) => Err(eyre!("environment variable [{LOCATION}] was not in valid unicode")),
        }
    }
}

pub(crate) mod gridfs {
    use azalia::{
        config::{env, merge::Merge},
        remi::{
            self,
            gridfs::mongodb::{
                bson::Document,
                options::{
                    Acknowledgment, AuthMechanism, ClientOptions, Credential, HedgedReadOptions, ReadConcern,
                    ReadPreference, ReadPreferenceOptions, SelectionCriteria, ServerAddress, TagSet, WriteConcern,
                },
            },
        },
        TRUTHY_REGEX,
    };
    use eyre::Context;
    use std::{env::VarError, time::Duration};

    pub const WRITE_CONCERN_TIMEOUT: &str = "UME_STORAGE_GRIDFS_WRITE_CONCERN_TIMEOUT";
    pub const WRITE_CONCERN_JOURNAL: &str = "UME_STORAGE_GRIDFS_WRITE_CONCERN_JOURNAL";
    pub const SELECTION_CRITERIA: &str = "UME_STORAGE_GRIDFS_SELECTION_CRITERIA";
    pub const WRITE_CONCERN: &str = "UME_STORAGE_GRIDFS_WRITE_CONCERN";
    pub const READ_CONCERN: &str = "UME_STORAGE_GRIDFS_READ_CONCERN";
    pub const CHUNK_SIZE: &str = "UME_STORAGE_GRIDFS_CHUNK_SIZE";
    pub const DATABASE: &str = "UME_STORAGE_GRIDFS_DATABASE";
    pub const BUCKET: &str = "UME_STORAGE_GRIDFS_BUCKET";

    pub const CREDENTIAL_MECHANISM_PROPERTIES: &str = "UME_STORAGE_GRIDFS_CREDENTIAL_MECHANISM_PROPERTIES";
    pub const CREDENTIAL_MECHANISM: &str = "UME_STORAGE_GRIDFS_CREDENTIAL_MECHANISM";
    pub const CREDENTIAL_SOURCE: &str = "UME_STORAGE_GRIDFS_CREDENTIAL_SOURCE";
    pub const CONNECT_TIMEOUT: &str = "UME_STORAGE_GRIDFS_CONNECT_TIMEOUT";
    pub const CREDENTIAL: &str = "UME_STORAGE_GRIDFS_CREDENTIALS";
    pub const APP_NAME: &str = "UME_STORAGE_GRIDFS_APP_NAME";
    pub const HOSTS: &str = "UME_STORAGE_GRIDFS_SERVERS";

    pub const MAX_STALENESS: &str = "UME_STORAGE_GRIDFS_READ_PREFERENCE_MAX_STALENESS";
    pub const TAG_SETS: &str = "UME_STORAGE_GRIDFS_READ_PREFERENCE_TAG_SETS";
    pub const HEDGE: &str = "UME_STORAGE_GRIDFS_READ_PREFERENCE_HEDGE";

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

    pub fn merge_config(me: &mut remi::gridfs::StorageConfig, other: remi::gridfs::StorageConfig) {
        merge_tuple!(me.selection_criteria, other.selection_criteria);
        merge_tuple!(me.write_concern, other.write_concern);
        merge_tuple!(me.read_concern, other.read_concern);
        merge_tuple!(me.chunk_size, other.chunk_size, copyable);

        // We only do a subset of merging of client options as if we only
        // did `me.client_options != other.client_options`, then `me.client_options`
        // will implicitly merge to `other.client_options` with different settings.
        {
            // Instead of `vec<...> == vec<...>`, we check if any hosts are not in the
            // current object and merge them with the right-hand side object.
            if !me
                .client_options
                .hosts
                .iter()
                .any(|host| other.client_options.hosts.contains(host))
            {
                me.client_options.hosts.merge(other.client_options.hosts);
            }

            merge_tuple!(me.client_options.credential, other.client_options.credential);
        }

        me.database.merge(other.database);
        me.bucket.merge(other.bucket);
    }

    pub fn create_config() -> eyre::Result<remi::gridfs::StorageConfig> {
        Ok(remi::gridfs::StorageConfig {
            selection_criteria: parse_selection_criteria()?,
            client_options: parse_client_options()?,
            write_concern: parse_write_concern()?,
            read_concern: parse_read_concern()?,
            chunk_size: match env!(CHUNK_SIZE) {
                Ok(value) => value
                    .parse()
                    .map(Some)
                    .with_context(|| format!("unable to parse `{value}` as a u32"))?,

                Err(std::env::VarError::NotPresent) => None,
                Err(_) => return Err(eyre!("received invalid UTF-8 for environment variable `${CHUNK_SIZE}`")),
            },

            database: env!(DATABASE, optional),
            bucket: env!(BUCKET, optional).unwrap_or("ume".to_owned()),
        })
    }

    ///////////                               CLIENT OPTIONS                                   \\\\\\\\\\\

    fn parse_client_options() -> eyre::Result<ClientOptions> {
        Ok(ClientOptions::builder()
            .app_name(env!(APP_NAME, optional))
            .connect_timeout(parse_connect_timeout()?)
            .hosts(parse_hosts()?)
            .credential(parse_credentials()?)
            .build())
    }

    fn parse_connect_timeout() -> eyre::Result<Option<Duration>> {
        match env!(CONNECT_TIMEOUT) {
            Ok(value) => Ok(Some(humantime::parse_duration(&value)?)),
            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(_)) => Err(eyre!(
                "environment variable `${CONNECT_TIMEOUT}` was not a valid unicode string"
            )),
        }
    }

    fn parse_hosts() -> eyre::Result<Vec<ServerAddress>> {
        match env!(HOSTS) {
            Ok(res) => {
                let mut values = Vec::new();
                for server in res.split(',') {
                    values.push(server.parse()?);
                }

                Ok(values)
            }

            Err(VarError::NotPresent) => Ok(vec!["localhost:27017".parse()?]),
            Err(VarError::NotUnicode(_)) => Err(eyre!("environment variable `${HOSTS}` was not valid unicode")),
        }
    }

    fn parse_credentials() -> eyre::Result<Option<Credential>> {
        match env!(CREDENTIAL) {
            Ok(value) => {
                let (user, pass) = value
                    .split_once(':')
                    .ok_or_else(|| {
                        eyre!("environment variable `${CREDENTIAL}` must be in the form of 'username:password'")
                    })
                    .context("if there is no username, but with a password, do: ':<password>'")
                    .context("if there is no password, but a username, do: '<username>:'")?;

                Ok(Some(
                    Credential::builder()
                        .username((!user.is_empty()).then_some(user.to_owned()))
                        .password((!pass.is_empty()).then_some(pass.to_owned()))
                        .source(env!(CREDENTIAL_SOURCE, optional))
                        .mechanism(match env!(CREDENTIAL_MECHANISM) {
                            Ok(value) => match &*value.to_ascii_lowercase() {
                                "scram-sha1" => Some(AuthMechanism::ScramSha1),
                                "scram-sha256" => Some(AuthMechanism::ScramSha256),
                                "x509" => Some(AuthMechanism::MongoDbX509),
                                "gssapi" => Some(AuthMechanism::Gssapi),
                                "plain" => Some(AuthMechanism::Plain),
                                _ => None,
                            },

                            Err(VarError::NotPresent) => None,
                            Err(VarError::NotUnicode(_)) => {
                                return Err(eyre!(
                                    "environment variable `${CREDENTIAL_MECHANISM}` was not in valid unicode"
                                ))
                            }
                        })
                        .mechanism_properties(match env!(CREDENTIAL_MECHANISM_PROPERTIES) {
                            Ok(value) => {
                                let mut doc = Document::new();
                                for item in value.split(',') {
                                    if let Some((key, value)) = item.split_once('=') {
                                        if value.contains('=') {
                                            continue;
                                        }

                                        doc.insert(key, value.to_owned());
                                    }
                                }

                                Some(doc)
                            }

                            Err(VarError::NotPresent) => None,
                            Err(VarError::NotUnicode(_)) => {
                                return Err(eyre!(
                                    "environment variable `${CREDENTIAL_MECHANISM_PROPERTIES}` was not in valid unicode"
                                ))
                            }
                        })
                        .build(),
                ))
            }

            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(_)) => Err(eyre!("environment variable `${CREDENTIAL}` was not in valid unicode")),
        }
    }

    ///////////                               WRITE CONCERN                                   \\\\\\\\\\\

    fn parse_write_concern() -> eyre::Result<Option<WriteConcern>> {
        match env!(WRITE_CONCERN) {
            Ok(value) => Ok(Some(
                WriteConcern::builder()
                    .journal(env!(WRITE_CONCERN_JOURNAL, |val| TRUTHY_REGEX.is_match(&val); or false))
                    .w(match value.parse::<u32>() {
                        Ok(value) => Acknowledgment::Nodes(value),
                        Err(_) => match value.as_str() {
                            "majority" => Acknowledgment::Majority,
                            s => Acknowledgment::Custom(s.to_owned()),
                        },
                    })
                    .w_timeout(match env!(WRITE_CONCERN_TIMEOUT) {
                        Ok(value) => Some(humantime::parse_duration(&value)?),
                        Err(VarError::NotPresent) => None,
                        Err(VarError::NotUnicode(_)) => {
                            return Err(eyre!(
                                "environment variable `${WRITE_CONCERN_TIMEOUT}` was not in valid unicode"
                            ))
                        }
                    })
                    .build(),
            )),

            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(_)) => {
                Err(eyre!("environment variable `${WRITE_CONCERN}` is not in valid unicode"))
            }
        }
    }

    ///////////                                READ CONCERN                                    \\\\\\\\\\\

    fn parse_read_concern() -> eyre::Result<Option<ReadConcern>> {
        match env!(READ_CONCERN) {
            Ok(val) => Ok(Some(match &*val.to_ascii_lowercase() {
                "majority" => ReadConcern::majority(),
                "linear" | "linearizable" => ReadConcern::linearizable(),
                "local" => ReadConcern::local(),
                "avaliable" => ReadConcern::available(),
                "snapshot" => ReadConcern::snapshot(),
                s => ReadConcern::custom(s),
            })),

            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(_)) => {
                Err(eyre!("environment variable `${WRITE_CONCERN}` is not in valid unicode"))
            }
        }
    }

    ///////////                             SELECTION CRITERIA                                  \\\\\\\\\\\

    fn parse_selection_criteria() -> eyre::Result<Option<SelectionCriteria>> {
        match env!(SELECTION_CRITERIA) {
            Ok(res) => match &*res.to_ascii_lowercase() {
                "primary" => Ok(Some(SelectionCriteria::ReadPreference(ReadPreference::Primary))),
                "secondary" => Ok(Some(SelectionCriteria::ReadPreference(ReadPreference::Secondary {
                    options: parse_read_preference_options().map(Some)?,
                }))),

                "primary-preferred" => Ok(Some(SelectionCriteria::ReadPreference(
                    ReadPreference::PrimaryPreferred {
                        options: parse_read_preference_options().map(Some)?,
                    },
                ))),

                "secondary-preferred" => Ok(Some(SelectionCriteria::ReadPreference(
                    ReadPreference::SecondaryPreferred {
                        options: parse_read_preference_options().map(Some)?,
                    },
                ))),

                "nearest" => Ok(Some(SelectionCriteria::ReadPreference(ReadPreference::Nearest {
                    options: parse_read_preference_options().map(Some)?,
                }))),

                input => Err(eyre!("unknown input [{input}] for environment variable `${SELECTION_CRITERIA}` | expected [primary, primary-preferred, secondary, secondary-preferred, nearest]"))
            },

            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(_)) => Err(eyre!("environment variable `${SELECTION_CRITERIA}` was not in valid unicode"))
        }
    }

    fn parse_read_preference_options() -> eyre::Result<ReadPreferenceOptions> {
        Ok(ReadPreferenceOptions::builder()
            .tag_sets(parse_tag_sets()?)
            .max_staleness(parse_max_staleness()?)
            .hedge(parse_hedge())
            .build())
    }

    fn parse_tag_sets() -> eyre::Result<Option<Vec<TagSet>>> {
        match env!(TAG_SETS) {
            Ok(value) => {
                let mut sets = Vec::new();
                for line in value.split(',') {
                    if let Some((key, value)) = line.split_once('=') {
                        if value.contains('=') {
                            continue;
                        }

                        sets.push(TagSet::from_iter([(key.into(), value.into())]));
                    }
                }

                Ok(Some(sets))
            }

            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(_)) => Err(eyre!("environment variable `${TAG_SETS}` was not in valid unicode")),
        }
    }

    fn parse_max_staleness() -> eyre::Result<Option<Duration>> {
        match env!(MAX_STALENESS) {
            Ok(res) => Ok(Some(humantime::parse_duration(&res)?)),
            Err(VarError::NotPresent) => Ok(None),
            Err(VarError::NotUnicode(_)) => Err(eyre!(
                "environment variable `${MAX_STALENESS}` was not in valid unicode"
            )),
        }
    }

    fn parse_hedge() -> Option<HedgedReadOptions> {
        if env!(HEDGE).is_err() {
            return None;
        }

        Some(
            HedgedReadOptions::builder()
                .enabled(env!(HEDGE, |val| TRUTHY_REGEX.is_match(&val); or false))
                .build(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use azalia::config::{expand_with, TryFromEnv};

    #[test]
    fn test_filesystem_configuration() {
        expand_with("UME_STORAGE_SERVICE", "filesystem", || {
            let config = Config::try_from_env().unwrap();
            let Config::Filesystem(_) = config else { unreachable!() };
        });

        expand_with("UME_STORAGE_SERVICE", "fs", || {
            let config = Config::try_from_env().unwrap();
            let Config::Filesystem(_) = config else { unreachable!() };
        });
    }

    // fn s3_config(f: impl FnOnce()) {
    //     let _ = EnvGuard::enter_with(s3::SECRET_ACCESS_KEY, "blah");
    //     let _ = EnvGuard::enter_with(s3::ACCESS_KEY_ID, "blah");

    //     expand_with("UME_STORAGE_SERVICE", "s3", f);
    // }

    // #[test]
    // fn test_s3_configuration() {
    //     s3_config(|| {
    //         expand_with(s3::ENABLE_SIGNER_V4_REQUESTS, "1", || {
    //             let config = Config::try_from_env().unwrap();
    //             let Config::S3(config) = config else { unreachable!() };

    //             assert!(config.enable_signer_v4_requests);
    //         });
    //     });
    // }

    // #[test]
    // fn test_azure_configuration() {}

    // #[test]
    // fn test_gridfs_configuration() {}
}
