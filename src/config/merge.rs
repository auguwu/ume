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

// this is used in noelware-rs' source code, but it's not available yet! so,
// I am going to re-use it for now, but references to `noelware-*` crates
// will be available soon.

pub mod strategy;

/// Trait that allows you to merge together two objects into one easily. This
/// is mainly used for the [`noelware-config`] crate to allow merges between
/// defaults, system environment variables, file-loading, and command-line arguments.
///
/// This can be used to your advantage to allow deep merging.
///
/// ## Example
/// ```ignore
/// # use noelware_merge::Merge;
/// #
/// pub struct MyWrapper(u64);
/// #
/// impl Merge for MyWrapper {
///     fn merge(&mut self, other: Self) {
///         *self.0 = other.0;
///     }
/// }
/// ```
///
/// [`noelware-config`]: https://crates.noelware.cloud/-/noelware-config/docs/latest
pub trait Merge {
    /// Does the merging all-together by modifying `self` from `other`.
    fn merge(&mut self, other: Self);
}

impl Merge for () {
    fn merge(&mut self, _other: Self) {
        // do nothing
    }
}

impl<T> Merge for Option<T> {
    fn merge(&mut self, mut other: Self) {
        if !self.is_some() {
            *self = other.take();
        }
    }
}

impl<T> Merge for Vec<T> {
    fn merge(&mut self, other: Self) {
        strategy::vec::extend(self, other);
    }
}

impl<K: std::hash::Hash + Eq, V> Merge for std::collections::HashMap<K, V> {
    fn merge(&mut self, other: Self) {
        self.extend(other);
    }
}

impl<V: std::hash::Hash + Eq> Merge for std::collections::HashSet<V> {
    fn merge(&mut self, other: Self) {
        self.extend(other);
    }
}

impl Merge for String {
    fn merge(&mut self, other: Self) {
        // fast path: don't merge if both are empty
        if self.is_empty() && other.is_empty() {
            return;
        }

        // fast path: copy other -> self if self is empty
        if self.is_empty() && !other.is_empty() {
            *self = other;
            return;
        }

        // slow path: compare strings at the end
        if *self != other {
            *self = other;
        }
    }
}

macro_rules! merge_unumbers {
    ($($ty:ty),*) => {
        $(
            impl Merge for $ty {
                fn merge(&mut self, other: Self) {
                    // fast-path: if both are 0, then don't do anything
                    if *self == 0 && other == 0 {
                        return;
                    }

                    // fast path: if self is 0 and other is not, then override
                    if *self == 0 && other > 0 {
                        *self = other;
                        return;
                    }

                    // slow path: compare
                    if *self != other {
                        *self = other;
                    }
                }
            }
        )*
    }
}

merge_unumbers!(u8, u16, u32, u64, u128, usize);

macro_rules! merge_numbers {
    ($($ty:ty),*) => {
        $(
            impl Merge for $ty {
                fn merge(&mut self, other: Self) {
                    // do comparsions
                    if *self != other {
                        *self = other;
                    }
                }
            }
        )*
    }
}

merge_numbers!(i8, i16, i32, i64, i128, isize);
