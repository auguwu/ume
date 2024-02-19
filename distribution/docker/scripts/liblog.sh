#!/usr/bin/env bash

# 🐻‍❄️💐 ume: Easy, self-hostable, and flexible image host made in Rust
# Copyright 2021-2024 Noel Towa <cutie@floofy.dev>
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

BLUE='\033[38;2;81;81;140m'
GREEN='\033[38;2;165;204;165m'
PINK='\033[38;2;241;204;209m'
RESET='\033[0m'
BOLD='\033[1m'
UNDERLINE='\033[4m'
RED='\033[38;166;76;76m'
YELLOW='\033[38;233;233;130m'

info() {
    timestamp=$(date +"%D ~ %r")
    printf "%b\\n" "${GREEN}${BOLD}info${RESET}  | ${PINK}${BOLD}${timestamp}${RESET} ~ $1"
}

debug() {
    local debug="${UME_DEBUG:-false}"
    shopt -s nocasematch
    timestamp=$(date +"%D ~ %r")

    if ! [[ "$debug" = "1" || "$debug" =~ ^(no|false)$ ]]; then
        printf "%b\\n" "${BLUE}${BOLD}debug${RESET} | ${PINK}${BOLD}${timestamp}${RESET} $1"
    fi
}

error() {
    timestamp=$(date +"%D ~ %r")
    printf "%b\\n" "${RED}${BOLD}error${RESET} | ${PINK}${BOLD}${timestamp}${RESET} $1"
}

warn() {
    timestamp=$(date +"%D ~ %r")
    printf "%b\\n" "${RED}${BOLD}warn${RESET}  | ${PINK}${BOLD}${timestamp}${RESET} $1"
}
