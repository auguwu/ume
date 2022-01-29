#!/bin/bash

# 💖 ume: Easy, self-hostable, and flexible image and file host, made in Go using MongoDB GridFS.
# Copyright (c) 2020-2022 Noel <cutie@floofy.dev>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

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
  timestamp=$(date +"%D ~%r")

  if ! [[ "$debug" = "1" || "$debug" =~ ^(no|false)$ ]]; then
    printf "%b\\n" "${BLUE}${BOLD}debug${RESET} | ${PINK}${BOLD}${timestamp}${RESET} $1"
  fi
}

error() {
  timestamp=$(date +"%D ~%r")
  printf "%b\\n" "${RED}${BOLD}error${RESET} | ${PINK}${BOLD}${timestamp}${RESET} $1"
}

warn() {
  timestamp=$(date +"%D ~%r")
  printf "%b\\n" "${RED}${BOLD}warn${RESET}  | ${PINK}${BOLD}${timestamp}${RESET} $1"
}
