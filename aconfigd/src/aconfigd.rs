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

use crate::storage_files_manager::StorageFilesManager;
use crate::utils::{read_pb_from_file, remove_file};
use crate::AconfigdError;
use aconfigd_protos::ProtoPersistStorageRecords;
use anyhow::anyhow;
use log::{log, Level};
use std::path::{Path, PathBuf};

// Aconfigd that is capable of doing both one shot storage file init and socket service
#[derive(Debug)]
pub struct Aconfigd {
    pub root_dir: PathBuf,
    pub persist_storage_records: PathBuf,
    pub storage_manager: StorageFilesManager,
}

impl Aconfigd {
    /// Constructor
    pub fn new(root_dir: &Path, records: &Path) -> Self {
        Self {
            root_dir: root_dir.to_path_buf(),
            persist_storage_records: records.to_path_buf(),
            storage_manager: StorageFilesManager::new(root_dir),
        }
    }

    /// Initialize mainline storage files
    pub fn initialize_mainline_storage(&mut self) -> Result<(), AconfigdError> {
        let boot_dir = self.root_dir.join("boot");
        let pb = read_pb_from_file::<ProtoPersistStorageRecords>(&self.persist_storage_records)?;
        for entry in pb.records.iter() {
            let boot_value_file = boot_dir.join(entry.container().to_owned() + ".val");
            let boot_info_file = boot_dir.join(entry.container().to_owned() + ".info");
            if boot_value_file.exists() {
                remove_file(&boot_value_file)?;
            }
            if boot_info_file.exists() {
                remove_file(&boot_info_file)?;
            }
            self.storage_manager.add_storage_files_from_pb(entry)?;
        }

        // get all the apex dirs to visit
        let mut dirs_to_visit = Vec::new();
        let apex_dir = PathBuf::from("/apex");
        for entry in std::fs::read_dir(&apex_dir).map_err(|errmsg| {
            AconfigdError::FailToReadDir(anyhow!("Fail to read /apex dir: {}", errmsg))
        })? {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if !path.is_dir() {
                        continue;
                    }
                    if let Some(base_name) = path.file_name() {
                        if let Some(dir_name) = base_name.to_str() {
                            if dir_name.starts_with('.') {
                                continue;
                            }
                            if dir_name.find('@').is_some() {
                                continue;
                            }
                            if dir_name == "sharedlibs" {
                                continue;
                            }
                            dirs_to_visit.push(dir_name.to_string());
                        }
                    }
                }
                Err(errmsg) => {
                    log!(Level::Warn, "failed to visit entry: {}", errmsg);
                }
            }
        }

        // initialize each container
        for container in dirs_to_visit.iter() {
            let etc_dir = apex_dir.join(container).join("etc");
            let default_package_map = etc_dir.join("package.map");
            let default_flag_map = etc_dir.join("flag.map");
            let default_flag_val = etc_dir.join("flag.val");
            let default_flag_info = etc_dir.join("flag.info");

            if !default_package_map.exists()
                || !default_flag_map.exists()
                || !default_flag_val.exists()
                || !default_flag_val.exists()
            {
                continue;
            }

            if std::fs::metadata(&default_flag_val)
                .map_err(|errmsg| {
                    AconfigdError::FailToGetFileMetadata(anyhow!(
                        "Fail to get file {} metadata: {}",
                        default_flag_val.display(),
                        errmsg
                    ))
                })?
                .len()
                == 0
            {
                continue;
            }

            self.storage_manager.add_or_update_container_storage_files(
                container,
                &default_package_map,
                &default_flag_map,
                &default_flag_val,
                &default_flag_info,
            )?;

            self.storage_manager
                .write_persist_storage_records_to_file(&self.persist_storage_records)?;

            self.storage_manager.create_storage_boot_copy(container)?;
        }

        Ok(())
    }
}
