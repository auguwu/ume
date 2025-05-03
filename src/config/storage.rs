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

use azalia::{
    config::{
        env::{self, TryFromEnv},
        merge::Merge,
    },
    remi,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const STORAGE_SERVICE: &str = "UME_STORAGE_SERVICE";

/// Represents the configuration for configuring the data storage where
/// ume will put all images in.
///
/// ## Examples
/// ### Filesystem
/// ```toml
/// [storage.filesystem]
/// directory = "/var/lib/noel/ume/images"
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
    type Error = eyre::Report;

    fn try_from_env() -> Result<Self, Self::Error> {
        crate::config::impl_enum_based_env_value!(STORAGE_SERVICE, {
            on match fail: |input| "environment variable `${}` is invalid: expected `filesystem`, `s3`, `azure`, or `gridfs`: received '{}' instead!" [STORAGE_SERVICE, input];

            "filesystem" | "fs" | "" => Ok(Config::Filesystem(remi::fs::StorageConfig {
                directory: env::try_parse_or(filesystem::DIRECTORY, || PathBuf::from("./data"))?
            }));

            "s3" => Ok(Config::S3(s3::create_config()?));
            "gridfs" => Ok(Config::Gridfs(gridfs::create_config()?));
            "azure" => Ok(Config::Azure(azure::create_config()?));
        })
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

pub(crate) mod filesystem {
    pub const DIRECTORY: &str = "UME_STORAGE_FILESYSTEM_DIRECTORY";
}

pub(crate) mod s3 {
    use crate::config::util;
    use azalia::{
        config::{
            env::{self, TryFromEnvValue},
            merge::Merge,
        },
        remi::{
            self,
            s3::aws::{
                config::Region,
                s3::types::{BucketCannedAcl, ObjectCannedAcl},
            },
        },
    };
    use std::{borrow::Cow, convert::Infallible};

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

    struct RegionEnv(Region);
    impl TryFromEnvValue for RegionEnv {
        type Error = Infallible;

        fn try_from_env_value(value: String) -> Result<Self, Self::Error> {
            Ok(RegionEnv(Region::new(value)))
        }
    }

    pub fn create_config() -> eyre::Result<remi::s3::StorageConfig> {
        Ok(remi::s3::StorageConfig {
            enable_signer_v4_requests: util::bool_env(ENABLE_SIGNER_V4_REQUESTS)?,
            enforce_path_access_style: util::bool_env(ENFORCE_PATH_ACCESS_STYLE)?,
            default_bucket_acl: util::env_from_str(DEFAULT_BUCKET_ACL, DEFAULT_BUCKET_CANNED_ACL).map(Some)?,
            default_object_acl: util::env_from_str(DEFAULT_OBJECT_ACL, DEFAULT_OBJECT_CANNED_ACL).map(Some)?,
            secret_access_key: env::try_parse(SECRET_ACCESS_KEY)?,
            access_key_id: env::try_parse(ACCESS_KEY_ID)?,
            app_name: env::try_parse_optional(APP_NAME)?,
            endpoint: env::try_parse_optional(ENDPOINT)?,
            prefix: env::try_parse_optional(PREFIX)?,
            region: env::try_parse_or_else::<_, RegionEnv>(REGION, RegionEnv(Region::new(Cow::Borrowed("us-east-1"))))
                .map(|s| Some(s.0))?,

            bucket: env::try_parse_optional(BUCKET)?.unwrap_or(String::from("ume")),
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
            container: env::try_parse_optional(CONTAINER)?.unwrap_or(String::from("ume")),
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
        crate::config::impl_enum_based_env_value!(CREDENTIAL, {
            on match fail: |input| "invalid input [{}] for `${}`: expected either `anonymous` (`anon` is accepted as well), \
                `sastoken` (`sas-token`, `sas_token` is accepted as well), \
                `accesskey` (`access-key` and `access_key` is accepted as well) \
                or `bearer`." [input, CREDENTIAL];

            "anonymous" | "anon" | "" => Ok(remi::azure::Credential::Anonymous);
            "accesskey" | "access-key" | "access_key" => Ok(remi::azure::Credential::AccessKey {
                account: env::try_parse(ACCESS_KEY_ACCOUNT)?,
                access_key: env::try_parse(ACCESS_KEY)?
            });

            "sastoken" | "sas-token" | "sas_token" => Ok(remi::azure::Credential::SASToken(env::try_parse(SAS_TOKEN)?));
            "bearer" => Ok(remi::azure::Credential::Bearer(env::try_parse(BEARER)?));
        })
    }

    fn create_location() -> eyre::Result<CloudLocation> {
        crate::config::impl_enum_based_env_value!(LOCATION, {
            on match fail: |input| "invalid input [{}] for `${}`: expected either: `public`, `china`, or `custom`." [input, LOCATION];

            "public" | "" => Ok(CloudLocation::Public(env::try_parse(ACCOUNT)?));
            "china" => Ok(CloudLocation::China(env::try_parse(ACCOUNT)?));
            "custom" => Ok(CloudLocation::Custom {
                account: env::try_parse(ACCOUNT)?,
                uri: env::try_parse(URI)?
            });
        })
    }
}

pub(crate) mod gridfs {
    use azalia::{
        config::{
            env::{self, TryFromEnvValue},
            merge::Merge,
        },
        remi::{
            self,
            gridfs::mongodb::{
                bson::{Bson, Document},
                options::{
                    Acknowledgment, AuthMechanism, ClientOptions, Credential, HedgedReadOptions, ReadConcern,
                    ReadPreference, ReadPreferenceOptions, SelectionCriteria, ServerAddress, TagSet, WriteConcern,
                },
            },
        },
    };
    use charted_core::ResultExt;
    use eyre::Context;
    use std::{
        collections::{BTreeMap, HashMap},
        convert::Infallible,
        str::FromStr,
        time::Duration,
    };

    use crate::config::util;

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
    pub const REPLICA_SET: &str = "UME_STORAGE_GRIDFS_REPLICA_SET";
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
            merge_tuple!(me.client_options.repl_set_name, other.client_options.repl_set_name);
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
            chunk_size: env::try_parse_optional(CHUNK_SIZE)?,
            database: env::try_parse_optional(DATABASE)?,
            bucket: env::try_parse_optional(BUCKET)?.unwrap_or(String::from("ume")),
        })
    }

    ///////////                               CLIENT OPTIONS                                   \\\\\\\\\\\

    fn parse_client_options() -> eyre::Result<ClientOptions> {
        Ok(ClientOptions::builder()
            .app_name(env::try_parse_optional(APP_NAME)?)
            .connect_timeout(parse_connect_timeout()?)
            .hosts(parse_hosts()?)
            .credential(parse_credentials()?)
            .repl_set_name(env::try_parse_optional(REPLICA_SET)?)
            .build())
    }

    fn parse_connect_timeout() -> eyre::Result<Option<Duration>> {
        match env::try_parse_optional::<_, String>(CONNECT_TIMEOUT) {
            Ok(Some(value)) => Ok(Some(charted_core::serde::Duration::from_str(&value)?.into())),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn parse_hosts() -> eyre::Result<Vec<ServerAddress>> {
        env::try_parse_or::<_, Vec<String>>(HOSTS, || vec!["localhost:27017".to_owned()])?
            .into_iter()
            .map(|v| <ServerAddress as FromStr>::from_str(&v).into_report())
            .collect::<eyre::Result<_>>()
    }

    fn parse_credentials() -> eyre::Result<Option<Credential>> {
        let Some(credential) = env::try_parse_optional::<_, String>(CREDENTIAL)? else {
            return Ok(None);
        };

        let (user, pass) = credential
            .trim()
            .split_once(':')
            .ok_or_else(|| {
                eyre!(
                    "environment variable `${}` must be in a form of 'username:password'",
                    CREDENTIAL
                )
            })
            .context("if there is no username but a password, use: ':<password>'")
            .context("if there is no password but a username, use: '<username>:'")?;

        Ok(Some(
            Credential::builder()
                .username((!user.is_empty()).then_some(user.to_owned()))
                .password((!pass.is_empty()).then_some(pass.to_owned()))
                .source(env::try_parse_optional(CREDENTIAL_SOURCE)?)
                .mechanism(crate::config::impl_enum_based_env_value!(CREDENTIAL_MECHANISM, {
                    on match fail: |input| "environment variable `${}` was invalid from given input: '{}'" [CREDENTIAL_MECHANISM, input];

                    "scram-sha1" => Some(AuthMechanism::ScramSha1);
                    "scram-sha256" => Some(AuthMechanism::ScramSha256);
                    "x509" => Some(AuthMechanism::MongoDbX509);
                    "gssapi" => Some(AuthMechanism::Gssapi);
                    "plain" => Some(AuthMechanism::Plain);
                    _ => None;
                }))
                .mechanism_properties({
                    let values = env::try_parse_or::<_, BTreeMap<String, String>>(CREDENTIAL_MECHANISM_PROPERTIES, BTreeMap::new)?;
                    values.into_iter().map(|(key, value)| (key, Bson::String(value))).collect::<Document>()
                })
                .build()
        ))
    }

    ///////////                               WRITE CONCERN                                   \\\\\\\\\\\

    fn parse_write_concern() -> eyre::Result<Option<WriteConcern>> {
        let Some(value) = env::try_parse_optional::<_, String>(WRITE_CONCERN)? else {
            return Ok(None);
        };

        Ok(Some(
            WriteConcern::builder()
                .journal(util::bool_env(WRITE_CONCERN_JOURNAL)?)
                .w(value.parse::<u32>().map(Acknowledgment::Nodes).unwrap_or_else(|_| {
                    match &*value.to_ascii_lowercase() {
                        "majority" => Acknowledgment::Majority,
                        s => Acknowledgment::Custom(s.to_owned()),
                    }
                }))
                .w_timeout(match env::try_parse_optional::<_, String>(WRITE_CONCERN_TIMEOUT) {
                    Ok(Some(value)) => Some(charted_core::serde::Duration::from_str(&value)?.into()),
                    Ok(None) => None,
                    Err(e) => return Err(e.into()),
                })
                .build(),
        ))
    }

    ///////////                                READ CONCERN                                    \\\\\\\\\\\

    fn parse_read_concern() -> eyre::Result<Option<ReadConcern>> {
        crate::config::impl_enum_based_env_value!(READ_CONCERN, {
            on match fail: |_input| "(this should never happen)";

            "majority" => Ok(Some(ReadConcern::majority()));
            "linear" | "linearizable" => Ok(Some(ReadConcern::linearizable()));
            "local" => Ok(Some(ReadConcern::local()));
            "avaliable" => Ok(Some(ReadConcern::available()));
            "snapshot" => Ok(Some(ReadConcern::snapshot()));
            s => Ok(Some(ReadConcern::custom(s)));
        })
    }

    ///////////                             SELECTION CRITERIA                                  \\\\\\\\\\\

    fn parse_selection_criteria() -> eyre::Result<Option<SelectionCriteria>> {
        crate::config::impl_enum_based_env_value!(SELECTION_CRITERIA, {
            on match fail: |input| "invalid value for `${}`: expected `primary`, `primary-preferred`, `secondary-preferred`, `secondary`, `nearest`: received '{}' instead" [SELECTION_CRITERIA, input];

            "primary" => Ok(Some(SelectionCriteria::ReadPreference(ReadPreference::Primary)));
            "secondary" => Ok(Some(SelectionCriteria::ReadPreference(ReadPreference::Secondary {
                options: parse_read_preference_options().map(Some)?,
            })));

            "primary-preferred" => Ok(Some(SelectionCriteria::ReadPreference(
                ReadPreference::PrimaryPreferred {
                    options: parse_read_preference_options().map(Some)?,
                },
            )));

            "secondary-preferred" => Ok(Some(SelectionCriteria::ReadPreference(
                ReadPreference::SecondaryPreferred {
                    options: parse_read_preference_options().map(Some)?,
                },
            )));

            "nearest" => Ok(Some(SelectionCriteria::ReadPreference(ReadPreference::Nearest {
                options: parse_read_preference_options().map(Some)?,
            })));
        })
    }

    fn parse_read_preference_options() -> eyre::Result<ReadPreferenceOptions> {
        Ok(ReadPreferenceOptions::builder()
            .tag_sets(parse_tag_sets()?)
            .max_staleness(parse_max_staleness()?)
            .hedge(parse_hedge()?)
            .build())
    }

    struct TagSetParse(Vec<TagSet>);
    impl TryFromEnvValue for TagSetParse {
        type Error = Infallible;

        fn try_from_env_value(value: String) -> Result<Self, Self::Error> {
            let mut sets = Vec::new();

            for elem in value.split(',') {
                let elements = elem.split(';');
                let mut map = HashMap::new();

                for line in elements {
                    if let Some((key, value)) = line.split_once('=') {
                        if value.contains('=') {
                            continue;
                        }

                        let key = String::try_from_env_value(key.to_owned()).unwrap();
                        let value = String::try_from_env_value(value.to_owned()).unwrap();

                        map.insert(key, value);
                    }
                }

                sets.push(map);
            }

            Ok(Self(sets))
        }
    }

    fn parse_tag_sets() -> eyre::Result<Option<Vec<TagSet>>> {
        env::try_parse_or::<_, TagSetParse>(TAG_SETS, || TagSetParse(Vec::new()))
            .map(|s| Some(s.0))
            .into_report()
    }

    fn parse_max_staleness() -> eyre::Result<Option<Duration>> {
        match env::try_parse_optional::<_, String>(MAX_STALENESS) {
            Ok(Some(value)) => Ok(Some(charted_core::serde::Duration::from_str(&value)?.into())),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn parse_hedge() -> eyre::Result<Option<HedgedReadOptions>> {
        let Some(_) = env::try_parse_optional::<_, String>(HEDGE)? else {
            return Ok(None);
        };

        Ok(Some(
            HedgedReadOptions::builder().enabled(util::bool_env(HEDGE)?).build(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Config;
    use azalia::config::env::{TryFromEnv, enter_with};

    #[test]
    fn test_filesystem_configuration() {
        enter_with("UME_STORAGE_SERVICE", "filesystem", || {
            let config = Config::try_from_env().unwrap();
            let Config::Filesystem(_) = config else { unreachable!() };
        });

        enter_with("UME_STORAGE_SERVICE", "fs", || {
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
