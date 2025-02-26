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

#![cfg(feature = "os-notifier")]

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{ops::Deref, str::FromStr};

/// Configuration for using the system's notifications for important messages
/// like file upload success or failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Config {
    Toggle(bool),
    Configurable(Settings),
}

impl Default for Config {
    fn default() -> Self {
        Config::Toggle(true)
    }
}

impl Config {
    pub const fn as_bool(&self) -> bool {
        match self {
            Config::Toggle(s) => *s,
            Config::Configurable(_) => true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// The lifetime of the notification. Usually not respected by the
    /// server implementation, so it might be useless.
    #[serde(default)]
    pub timeout: Timeout,
}

/// Newtype wrapper for [`notify_rust::Timeout`] to support `serde`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Timeout(notify_rust::Timeout);
impl From<Timeout> for notify_rust::Timeout {
    fn from(value: Timeout) -> Self {
        value.0
    }
}

impl Deref for Timeout {
    type Target = notify_rust::Timeout;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Serialize for Timeout {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.0 {
            notify_rust::Timeout::Default => serializer.serialize_str("default"),
            notify_rust::Timeout::Never => serializer.serialize_str("never"),
            notify_rust::Timeout::Milliseconds(ms) => serializer.serialize_u32(ms),
        }
    }
}

impl<'de> Deserialize<'de> for Timeout {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de;

        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = Timeout;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("either \"never\", \"default\", or any uint32 value representing milliseconds")
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                notify_rust::Timeout::from_str(&v)
                    .map(Timeout)
                    .map_err(de::Error::custom)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                de::Visitor::visit_string(self, v.to_owned())
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(Timeout(notify_rust::Timeout::Milliseconds(v)))
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}
