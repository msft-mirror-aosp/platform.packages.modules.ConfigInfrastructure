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

use std::path::PathBuf;
use tempfile::{tempdir, TempDir};

/// Container mockup
pub(crate) struct ContainerMock {
    pub tmp_dir: TempDir,
    pub name: String,
    pub package_map: PathBuf,
    pub flag_map: PathBuf,
    pub flag_val: PathBuf,
    pub flag_info: PathBuf,
}

/// Implementation for container mockup
impl ContainerMock {
    pub(crate) fn new() -> Self {
        let tmp_dir = tempdir().unwrap();
        let package_map = tmp_dir.path().join("package.map");
        let flag_map = tmp_dir.path().join("flag.map");
        let flag_val = tmp_dir.path().join("flag.val");
        let flag_info = tmp_dir.path().join("flag.info");
        std::fs::copy("./tests/data/package.map", &package_map).unwrap();
        std::fs::copy("./tests/data/flag.map", &flag_map).unwrap();
        std::fs::copy("./tests/data/flag.val", &flag_val).unwrap();
        std::fs::copy("./tests/data/flag.info", &flag_info).unwrap();
        Self { tmp_dir, name: String::from("mockup"), package_map, flag_map, flag_val, flag_info }
    }
}

/// Implement drop trait for ContainerMock
impl Drop for ContainerMock {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.tmp_dir).unwrap();
    }
}

/// Storage root dir mockup
pub(crate) struct StorageRootDirMock {
    pub tmp_dir: TempDir,
    pub flags_dir: PathBuf,
    pub maps_dir: PathBuf,
    pub boot_dir: PathBuf,
}

/// Implementation for storage root dir mockup
impl StorageRootDirMock {
    pub(crate) fn new() -> Self {
        let tmp_dir = tempdir().unwrap();
        let flags_dir = tmp_dir.path().join("flags");
        let maps_dir = tmp_dir.path().join("maps");
        let boot_dir = tmp_dir.path().join("boot");
        std::fs::create_dir(&flags_dir).unwrap();
        std::fs::create_dir(&maps_dir).unwrap();
        std::fs::create_dir(&boot_dir).unwrap();
        Self { tmp_dir, flags_dir, maps_dir, boot_dir }
    }
}

/// Implement drop trait for StorageRootDirMock
impl Drop for StorageRootDirMock {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.tmp_dir).unwrap();
    }
}
