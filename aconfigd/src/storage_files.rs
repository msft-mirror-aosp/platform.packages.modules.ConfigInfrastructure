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

use crate::AconfigdError;
use aconfig_storage_file::FlagValueType;
use aconfig_storage_read_api::{
    get_flag_read_context, get_package_read_context, get_storage_file_version, map_file,
};
use aconfig_storage_write_api::map_mutable_storage_file;
use aconfigd_protos::ProtoPersistStorageRecord;
use anyhow::anyhow;
use memmap2::{Mmap, MmapMut};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

// In memory data structure for storage file locations for each container
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct StorageRecord {
    pub version: u32,
    pub container: String,            // container name
    pub default_package_map: PathBuf, // default package map file
    pub default_flag_map: PathBuf,    // default flag map file
    pub default_flag_val: PathBuf,    // default flag val file
    pub default_flag_info: PathBuf,   // default flag info file
    pub persist_package_map: PathBuf, // persist package.map file
    pub persist_flag_map: PathBuf,    // persist flag.map file
    pub persist_flag_val: PathBuf,    // persist flag.val file
    pub persist_flag_info: PathBuf,   // persist flag.info file
    pub local_overrides: PathBuf,     // local overrides pb file
    pub boot_flag_val: PathBuf,       // boot flag.val file
    pub boot_flag_info: PathBuf,      // boot flag.info file
    pub digest: String,               // hash for all default storage files
}

// Storage files for a particular container
#[derive(Debug)]
pub(crate) struct StorageFiles {
    pub storage_record: StorageRecord,
    pub package_map: Option<Mmap>,
    pub flag_map: Option<Mmap>,
    pub flag_val: Option<Mmap>,             // default flag value file
    pub boot_flag_val: Option<Mmap>,        // boot flag value file
    pub boot_flag_info: Option<Mmap>,       // boot flag info file
    pub persist_flag_val: Option<MmapMut>,  // persist flag value file
    pub persist_flag_info: Option<MmapMut>, // persist flag info file
}

// Compare two options of mmap/mmapmut
fn same_mmap_contents<T: std::ops::Deref<Target = [u8]>>(
    opt_a: &Option<T>,
    opt_b: &Option<T>,
) -> bool {
    match (opt_a, opt_b) {
        (Some(map_a), Some(map_b)) => map_a[..] == map_b[..],
        (None, None) => true,
        _ => false,
    }
}

impl PartialEq for StorageFiles {
    fn eq(&self, other: &Self) -> bool {
        self.storage_record == other.storage_record
            && same_mmap_contents(&self.package_map, &other.package_map)
            && same_mmap_contents(&self.flag_map, &other.flag_map)
            && same_mmap_contents(&self.flag_val, &other.flag_val)
            && same_mmap_contents(&self.boot_flag_val, &other.boot_flag_val)
            && same_mmap_contents(&self.boot_flag_info, &other.boot_flag_info)
            && same_mmap_contents(&self.persist_flag_val, &other.persist_flag_val)
            && same_mmap_contents(&self.persist_flag_info, &other.persist_flag_info)
    }
}

// Package and flag query context
#[derive(PartialEq, Debug)]
pub(crate) struct PackageFlagContext {
    pub package: String,
    pub flag: String,
    pub package_exists: bool,
    pub flag_exists: bool,
    pub value_type: FlagValueType,
    pub flag_index: u32,
}

/// TODO: temp imlementation to copy file, to be replaced by util one
pub fn copy_file(src: &Path, dst: &Path, mode: u32) -> Result<(), AconfigdError> {
    std::fs::copy(src, dst).map_err(|errmsg| {
        AconfigdError::FailToCopyFile(anyhow!(
            "Failed to copy file from {} to {}: {}",
            src.display(),
            dst.display(),
            errmsg
        ))
    })?;
    let perms = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(dst, perms).map_err(|errmsg| {
        AconfigdError::FailToUpdateFilePerm(anyhow!(
            "Failed to set file permission to 0444 for {}: {}",
            dst.display(),
            errmsg
        ))
    })?;
    Ok(())
}

impl StorageFiles {
    /// Constructor from a container
    pub fn from_container(
        container: &str,
        package_map: &Path,
        flag_map: &Path,
        flag_val: &Path,
        flag_info: &Path,
        root_dir: &Path,
    ) -> Result<Self, AconfigdError> {
        let version =
            get_storage_file_version(&flag_val.display().to_string()).map_err(|errmsg| {
                AconfigdError::FailToParse(anyhow!(
                    "Failed to get file version from {} : {}",
                    flag_val.display(),
                    errmsg
                ))
            })?;

        let record = StorageRecord {
            version,
            container: container.to_string(),
            default_package_map: package_map.to_path_buf(),
            default_flag_map: flag_map.to_path_buf(),
            default_flag_val: flag_val.to_path_buf(),
            default_flag_info: flag_info.to_path_buf(),
            persist_package_map: root_dir
                .join("maps")
                .join(container.to_string() + ".package.map"),
            persist_flag_map: root_dir
                .join("maps")
                .join(container.to_string() + ".flag.map"),
            persist_flag_val: root_dir.join("flags").join(container.to_string() + ".val"),
            persist_flag_info: root_dir.join("flags").join(container.to_string() + ".info"),
            local_overrides: root_dir
                .join("flags")
                .join(container.to_string() + "_local_overrides.pb"),
            boot_flag_val: root_dir.join("boot").join(container.to_string() + ".val"),
            boot_flag_info: root_dir.join("boot").join(container.to_string() + ".info"),
            digest: String::new(),
        };

        copy_file(package_map, &record.persist_package_map, 0o444)?;
        copy_file(flag_map, &record.persist_flag_map, 0o444)?;
        copy_file(flag_val, &record.persist_flag_val, 0o644)?;
        copy_file(flag_info, &record.persist_flag_info, 0o644)?;

        let files = Self {
            storage_record: record,
            package_map: None,
            flag_map: None,
            flag_val: None,
            boot_flag_val: None,
            boot_flag_info: None,
            persist_flag_val: None,
            persist_flag_info: None,
        };

        Ok(files)
    }

    /// Constructor from a pb record
    pub fn from_pb(pb: &ProtoPersistStorageRecord, root_dir: &Path) -> Self {
        let record = StorageRecord {
            version: pb.version(),
            container: pb.container().to_string(),
            default_package_map: PathBuf::from(pb.package_map()),
            default_flag_map: PathBuf::from(pb.flag_map()),
            default_flag_val: PathBuf::from(pb.flag_val()),
            default_flag_info: PathBuf::from(pb.flag_info()),
            persist_package_map: root_dir
                .join("maps")
                .join(pb.container().to_string() + ".package.map"),
            persist_flag_map: root_dir
                .join("maps")
                .join(pb.container().to_string() + ".flag.map"),
            persist_flag_val: root_dir
                .join("flags")
                .join(pb.container().to_string() + ".val"),
            persist_flag_info: root_dir
                .join("flags")
                .join(pb.container().to_string() + ".info"),
            local_overrides: root_dir
                .join("flags")
                .join(pb.container().to_string() + "_local_overrides.pb"),
            boot_flag_val: root_dir
                .join("boot")
                .join(pb.container().to_string() + ".val"),
            boot_flag_info: root_dir
                .join("boot")
                .join(pb.container().to_string() + ".info"),
            digest: pb.digest().to_string(),
        };

        Self {
            storage_record: record,
            package_map: None,
            flag_map: None,
            flag_val: None,
            boot_flag_val: None,
            boot_flag_info: None,
            persist_flag_val: None,
            persist_flag_info: None,
        }
    }

    /// Get immutable file mapping of a file.
    ///
    /// # Safety
    ///
    /// The memory mapped file may have undefined behavior if there are writes to the underlying
    /// file after being mapped. Ensure no writes can happen to the underlying file that is memory
    /// mapped while this mapping stays alive to guarantee safety.
    unsafe fn get_immutable_file_mapping(file_path: &Path) -> Result<Mmap, AconfigdError> {
        // SAFETY: As per the safety comment, there are no other writes to the underlying file.
        unsafe {
            map_file(&file_path.display().to_string()).map_err(|errmsg| {
                AconfigdError::FailToMap(anyhow!(
                    "Failed to map file {} : {}",
                    file_path.display(),
                    errmsg
                ))
            })
        }
    }

    /// Get package map memory mapping.
    fn get_package_map(&mut self) -> Result<&Mmap, AconfigdError> {
        if self.package_map.is_none() {
            // SAFETY: Here it is safe as package map files are always read only.
            unsafe {
                self.package_map = Some(Self::get_immutable_file_mapping(
                    &self.storage_record.persist_package_map,
                )?);
            }
        }
        self.package_map
            .as_ref()
            .ok_or(AconfigdError::FailToMap(anyhow!(
                "Failed to map file {}",
                &self.storage_record.persist_package_map.display()
            )))
    }

    /// Get flag map memory mapping.
    fn get_flag_map(&mut self) -> Result<&Mmap, AconfigdError> {
        if self.flag_map.is_none() {
            // SAFETY: Here it is safe as flag map files are always read only.
            unsafe {
                self.flag_map = Some(Self::get_immutable_file_mapping(
                    &self.storage_record.persist_flag_map,
                )?);
            }
        }
        self.flag_map
            .as_ref()
            .ok_or(AconfigdError::FailToMap(anyhow!(
                "Failed to map file {}",
                &self.storage_record.persist_flag_map.display()
            )))
    }

    /// Get default flag value memory mapping.
    fn get_flag_val(&mut self) -> Result<&Mmap, AconfigdError> {
        if self.flag_val.is_none() {
            // SAFETY: Here it is safe as default flag value files are always read only.
            unsafe {
                self.flag_val = Some(Self::get_immutable_file_mapping(
                    &self.storage_record.default_flag_val,
                )?);
            }
        }
        self.flag_val
            .as_ref()
            .ok_or(AconfigdError::FailToMap(anyhow!(
                "Failed to map file {}",
                &self.storage_record.default_flag_val.display()
            )))
    }

    /// Get boot flag value memory mapping.
    ///
    /// # Safety
    ///
    /// The memory mapped file may have undefined behavior if there are writes to the underlying
    /// file after being mapped. Ensure no writes can happen to the underlying file that is memory
    /// mapped while this mapping stays alive to guarantee safety.
    unsafe fn get_boot_flag_val(&mut self) -> Result<&Mmap, AconfigdError> {
        if self.boot_flag_val.is_none() {
            // SAFETY: As per the safety comment, there are no other writes to the underlying file.
            unsafe {
                self.boot_flag_val = Some(Self::get_immutable_file_mapping(
                    &self.storage_record.boot_flag_val,
                )?);
            }
        }
        self.boot_flag_val
            .as_ref()
            .ok_or(AconfigdError::FailToMap(anyhow!(
                "Failed to map file {}",
                &self.storage_record.boot_flag_val.display()
            )))
    }

    /// Get boot flag info memory mapping.
    ///
    /// # Safety
    ///
    /// The memory mapped file may have undefined behavior if there are writes to the underlying
    /// file after being mapped. Ensure no writes can happen to the underlying file that is memory
    /// mapped while this mapping stays alive to guarantee safety.
    unsafe fn get_boot_flag_info(&mut self) -> Result<&Mmap, AconfigdError> {
        if self.boot_flag_info.is_none() {
            // SAFETY: As per the safety comment, there are no other writes to the underlying file.
            unsafe {
                self.boot_flag_info = Some(Self::get_immutable_file_mapping(
                    &self.storage_record.boot_flag_info,
                )?);
            }
        }
        self.boot_flag_info
            .as_ref()
            .ok_or(AconfigdError::FailToMap(anyhow!(
                "Failed to map file {}",
                &self.storage_record.boot_flag_info.display()
            )))
    }

    /// Get mutable file mapping of a file.
    ///
    /// # Safety
    ///
    /// The memory mapped file may have undefined behavior if there are writes to this
    /// file not thru this memory mapped file or there are concurrent writes to this
    /// memory mapped file. Ensure all writes to the underlying file are thru this memory
    /// mapped file and there are no concurrent writes.
    unsafe fn get_mutable_file_mapping(file_path: &Path) -> Result<MmapMut, AconfigdError> {
        // SAFETY: As per the safety comment, there are no other writes to the underlying file.
        unsafe {
            map_mutable_storage_file(&file_path.display().to_string()).map_err(|errmsg| {
                AconfigdError::FailToMap(anyhow!(
                    "Failed to map mutable file {} : {}",
                    file_path.display(),
                    errmsg
                ))
            })
        }
    }

    /// Get persist flag value memory mapping.
    fn get_persist_flag_val(&mut self) -> Result<&MmapMut, AconfigdError> {
        if self.persist_flag_val.is_none() {
            // SAFETY: safety is ensured that all writes to the persist file is thru this
            // memory mapping, and there are no concurrent writes
            unsafe {
                self.persist_flag_val = Some(Self::get_mutable_file_mapping(
                    &self.storage_record.persist_flag_val,
                )?);
            }
        }
        self.persist_flag_val
            .as_ref()
            .ok_or(AconfigdError::FailToMap(anyhow!(
                "Failed to map file {}",
                &self.storage_record.persist_flag_val.display()
            )))
    }

    /// Get persist flag info memory mapping.
    fn get_persist_flag_info(&mut self) -> Result<&MmapMut, AconfigdError> {
        if self.persist_flag_info.is_none() {
            // SAFETY: safety is ensured that all writes to the persist file is thru this
            // memory mapping, and there are no concurrent writes
            unsafe {
                self.persist_flag_info = Some(Self::get_mutable_file_mapping(
                    &self.storage_record.persist_flag_info,
                )?);
            }
        }
        self.persist_flag_info
            .as_ref()
            .ok_or(AconfigdError::FailToMap(anyhow!(
                "Failed to map file {}",
                &self.storage_record.persist_flag_info.display()
            )))
    }

    /// Get storage record
    pub fn storage_record(&self) -> &StorageRecord {
        &self.storage_record
    }

    /// Has boot copy
    pub fn has_boot_copy(&self) -> bool {
        Path::new(&self.storage_record.boot_flag_val).exists()
            && Path::new(&self.storage_record.boot_flag_info).exists()
    }

    /// Get package and flag query context
    pub fn get_package_flag_context(
        &mut self,
        package: &str,
        flag: &str,
    ) -> Result<PackageFlagContext, AconfigdError> {
        let mut context = PackageFlagContext {
            package: package.to_string(),
            flag: flag.to_string(),
            package_exists: false,
            flag_exists: false,
            value_type: FlagValueType::Boolean,
            flag_index: 0,
        };

        if package.is_empty() {
            return Ok(context);
        }

        let package_context =
            get_package_read_context(self.get_package_map()?, package).map_err(|errmsg| {
                AconfigdError::FailToParse(anyhow!(
                    "Failed to get package context for {} in container {}: {}",
                    package,
                    self.storage_record.container,
                    errmsg
                ))
            })?;

        if let Some(pkg) = package_context {
            context.package_exists = true;
            if flag.is_empty() {
                return Ok(context);
            }

            let flag_context = get_flag_read_context(self.get_flag_map()?, pkg.package_id, flag)
                .map_err(|errmsg| {
                    AconfigdError::FailToParse(anyhow!(
                        "Failed to get flag context for {}.{} in container {}: {}",
                        package,
                        flag,
                        self.storage_record.container,
                        errmsg
                    ))
                })?;

            if let Some(flg) = flag_context {
                context.flag_exists = true;
                context.value_type = FlagValueType::try_from(flg.flag_type).map_err(|errmsg| {
                    AconfigdError::InvalidFlagValueType(anyhow!(
                        "Invalid flag value type for {}.{} in container {}: {}",
                        package,
                        flag,
                        self.storage_record.container,
                        errmsg
                    ))
                })?;
                context.flag_index = pkg.boolean_start_index + flg.flag_index as u32;
            }
        }

        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{ContainerMock, StorageRootDirMock};
    use std::fs::File;
    use std::io::Read;

    fn has_same_content(file_one: &Path, file_two: &Path) -> bool {
        assert!(file_one.exists());
        assert!(file_two.exists());

        let mut f1 = File::open(file_one).unwrap();
        let mut b1 = Vec::new();
        f1.read_to_end(&mut b1).unwrap();

        let mut f2 = File::open(file_two).unwrap();
        let mut b2 = Vec::new();
        f2.read_to_end(&mut b2).unwrap();

        b1 == b2
    }

    fn create_mock_storage_files(
        container: &ContainerMock,
        root_dir: &StorageRootDirMock,
    ) -> StorageFiles {
        StorageFiles::from_container(
            &container.name,
            &container.package_map,
            &container.flag_map,
            &container.flag_val,
            &container.flag_info,
            &root_dir.tmp_dir.path(),
        )
        .unwrap()
    }

    #[test]
    fn test_create_storage_file_from_container() {
        let container = ContainerMock::new();
        let root_dir = StorageRootDirMock::new();
        let storage_files = create_mock_storage_files(&container, &root_dir);

        let expected_record = StorageRecord {
            version: 1,
            container: String::from("mockup"),
            default_package_map: container.package_map.clone(),
            default_flag_map: container.flag_map.clone(),
            default_flag_val: container.flag_val.clone(),
            default_flag_info: container.flag_info.clone(),
            persist_package_map: root_dir.maps_dir.join("mockup.package.map"),
            persist_flag_map: root_dir.maps_dir.join("mockup.flag.map"),
            persist_flag_val: root_dir.flags_dir.join("mockup.val"),
            persist_flag_info: root_dir.flags_dir.join("mockup.info"),
            local_overrides: root_dir.flags_dir.join("mockup_local_overrides.pb"),
            boot_flag_val: root_dir.boot_dir.join("mockup.val"),
            boot_flag_info: root_dir.boot_dir.join("mockup.info"),
            digest: String::new(),
        };

        let expected_storage_files = StorageFiles {
            storage_record: expected_record,
            package_map: None,
            flag_map: None,
            flag_val: None,
            boot_flag_val: None,
            boot_flag_info: None,
            persist_flag_val: None,
            persist_flag_info: None,
        };

        assert_eq!(storage_files, expected_storage_files);

        assert!(has_same_content(
            &container.package_map,
            &storage_files.storage_record.persist_package_map
        ));
        assert!(has_same_content(
            &container.flag_map,
            &storage_files.storage_record.persist_flag_map
        ));
        assert!(has_same_content(
            &container.flag_val,
            &storage_files.storage_record.persist_flag_val
        ));
        assert!(has_same_content(
            &container.flag_info,
            &storage_files.storage_record.persist_flag_info
        ));
    }

    #[test]
    fn test_create_storage_file_from_pb() {
        let mut pb = ProtoPersistStorageRecord::new();
        pb.set_version(123);
        pb.set_container(String::from("some_container"));
        pb.set_package_map(String::from("some_package_map"));
        pb.set_flag_map(String::from("some_flag_map"));
        pb.set_flag_val(String::from("some_flag_val"));
        pb.set_flag_info(String::from("some_flag_info"));
        pb.set_digest(String::from("abc"));

        let root_dir = StorageRootDirMock::new();
        let storage_files = StorageFiles::from_pb(&pb, &root_dir.tmp_dir.path());

        let expected_record = StorageRecord {
            version: 123,
            container: String::from("some_container"),
            default_package_map: PathBuf::from("some_package_map"),
            default_flag_map: PathBuf::from("some_flag_map"),
            default_flag_val: PathBuf::from("some_flag_val"),
            default_flag_info: PathBuf::from("some_flag_info"),
            persist_package_map: root_dir.maps_dir.join("some_container.package.map"),
            persist_flag_map: root_dir.maps_dir.join("some_container.flag.map"),
            persist_flag_val: root_dir.flags_dir.join("some_container.val"),
            persist_flag_info: root_dir.flags_dir.join("some_container.info"),
            local_overrides: root_dir.flags_dir.join("some_container_local_overrides.pb"),
            boot_flag_val: root_dir.boot_dir.join("some_container.val"),
            boot_flag_info: root_dir.boot_dir.join("some_container.info"),
            digest: String::from("abc"),
        };

        let expected_storage_files = StorageFiles {
            storage_record: expected_record,
            package_map: None,
            flag_map: None,
            flag_val: None,
            boot_flag_val: None,
            boot_flag_info: None,
            persist_flag_val: None,
            persist_flag_info: None,
        };

        assert_eq!(storage_files, expected_storage_files);
    }

    #[test]
    fn test_storage_record() {
        let container = ContainerMock::new();
        let root_dir = StorageRootDirMock::new();
        let storage_files = create_mock_storage_files(&container, &root_dir);
        assert_eq!(
            &storage_files.storage_record,
            storage_files.storage_record()
        );
    }

    #[test]
    fn test_has_boot_copy() {
        let container = ContainerMock::new();
        let root_dir = StorageRootDirMock::new();
        let storage_files = create_mock_storage_files(&container, &root_dir);
        assert!(!storage_files.has_boot_copy());
        let record = storage_files.storage_record();
        copy_file(&record.default_flag_val, &record.boot_flag_val, 0o444).unwrap();
        assert!(!storage_files.has_boot_copy());
        copy_file(&record.default_flag_info, &record.boot_flag_info, 0o444).unwrap();
        assert!(storage_files.has_boot_copy());
    }

    #[test]
    fn test_get_package_flag_context() {
        let container = ContainerMock::new();
        let root_dir = StorageRootDirMock::new();
        let mut storage_files = create_mock_storage_files(&container, &root_dir);

        let mut context = PackageFlagContext {
            package: String::from("not_exist"),
            flag: String::new(),
            package_exists: false,
            flag_exists: false,
            value_type: FlagValueType::Boolean,
            flag_index: 0,
        };
        let mut actual_context = storage_files
            .get_package_flag_context("not_exist", "")
            .unwrap();
        assert_eq!(context, actual_context);

        context.package = String::from("com.android.aconfig.storage.test_1");
        context.package_exists = true;
        actual_context = storage_files
            .get_package_flag_context("com.android.aconfig.storage.test_1", "")
            .unwrap();
        assert_eq!(context, actual_context);

        context.flag = String::from("not_exist");
        actual_context = storage_files
            .get_package_flag_context("com.android.aconfig.storage.test_1", "not_exist")
            .unwrap();
        assert_eq!(context, actual_context);

        context.flag = String::from("enabled_rw");
        context.flag_exists = true;
        context.flag_index = 2;
        actual_context = storage_files
            .get_package_flag_context("com.android.aconfig.storage.test_1", "enabled_rw")
            .unwrap();
        assert_eq!(context, actual_context);

        context.package = String::from("com.android.aconfig.storage.test_2");
        context.flag = String::from("disabled_rw");
        context.flag_index = 3;
        actual_context = storage_files
            .get_package_flag_context("com.android.aconfig.storage.test_2", "disabled_rw")
            .unwrap();
        assert_eq!(context, actual_context);
    }
}
