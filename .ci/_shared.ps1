# vendored from:
# https://github.com/charted-dev/charted/blob/fd8ee07a34bb15e3bee3f3f74d5736c460a83154/src/ci/_shared.ps1

# 🐻‍❄️📦 charted-server: Free, open source, and reliable Helm Chart registry made in Rust
# Copyright 2022-2025 Noelware, LLC. <team@noelware.org>
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#    http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

function IsContinousIntegration {
    return $null -ne $env:CI || IsGitHubActions
}

function IsGitHubActions {
    return $null -ne $env:GITHUB_ACTIONS
}

function StartGroup {
    Param(
        [Parameter(Mandatory=$true, Position=0)]
        [string]$Label
    )

    if (IsGitHubActions) {
        Write-Host "::group::$Label"
    } else {
        Write-Host "~>     $label"
    }
}

function EndGroup {
    if (IsGitHubActions) {
        Write-Host "::endgroup::"
    }
}
