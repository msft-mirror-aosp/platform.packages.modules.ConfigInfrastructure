/*
 * Copyright (C) 2024 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! `aconfig_mainline` is a crate that defines library functions that are needed by
//! aconfig daemon for mainline (aconfigd-mainline binary).

pub mod storage_files;

#[cfg(test)]
mod test_utils;

/// aconfigd-mainline error
#[non_exhaustive]
#[derive(thiserror::Error, Debug)]
pub enum AconfigdError {
    #[error("invalid command")]
    InvalidCommand(#[source] anyhow::Error),

    #[error("fail to parse storage file")]
    FailToParse(#[source] anyhow::Error),

    #[error("fail to map storage file")]
    FailToMap(#[source] anyhow::Error),

    #[error("invalid flag value type")]
    InvalidFlagValueType(#[source] anyhow::Error),

    #[error("failed to modify file permission")]
    FailToUpdateFilePerm(#[source] anyhow::Error),

    #[error("failed to copy file")]
    FailToCopyFile(#[source] anyhow::Error),
}
