# 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
# Copyright 2021-2025 Noel Towa <cutie@floofy.dev>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# This is the default configuration file when running `ume` from the Docker
# images [ghcr.io/auguwu/ume or auguwu/ume].
#
# this is a placeholder uploader key, it is recommended that you
# use a random generated one than this one.

# (this can be overwritten with the `UME_UPLOADER_KEY` environment variable
# or use `-e UME_UPLOADER_KEY=<some key here>` when running with Docker and
# this won't be used)
uploader_key = "a uploader key that is here because no one has bothered to update this, please update this or you will be laughed at"

storage "filesystem" {
  directory = "/var/lib/noel/ume/data"
}
