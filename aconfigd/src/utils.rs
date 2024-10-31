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
use anyhow::anyhow;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Set file permission
pub(crate) fn set_file_permission(file: &Path, mode: u32) -> Result<(), AconfigdError> {
    let perms = std::fs::Permissions::from_mode(mode);
    std::fs::set_permissions(file, perms).map_err(|errmsg| {
        AconfigdError::FailToUpdateFilePerm(anyhow!(
            "Failed to set file permission to 0444 for {}: {}",
            file.display(),
            errmsg
        ))
    })?;
    Ok(())
}

/// Copy file
pub(crate) fn copy_file(src: &Path, dst: &Path, mode: u32) -> Result<(), AconfigdError> {
    std::fs::copy(src, dst).map_err(|errmsg| {
        AconfigdError::FailToCopyFile(anyhow!(
            "Failed to copy file from {} to {}: {}",
            src.display(),
            dst.display(),
            errmsg
        ))
    })?;
    set_file_permission(dst, mode)
}

/// Remove file
pub(crate) fn remove_file(src: &Path) -> Result<(), AconfigdError> {
    std::fs::remove_file(src).map_err(|errmsg| {
        AconfigdError::FailToRemoveFile(anyhow!(
            "Fail to remove file {}: {}",
            src.display(),
            errmsg
        ))
    })
}

/// Read pb from file
pub(crate) fn read_pb_from_file<T: protobuf::Message>(file: &Path) -> Result<T, AconfigdError> {
    if !Path::new(file).exists() {
        return Ok(T::new());
    }

    let data = std::fs::read(file).map_err(|errmsg| {
        AconfigdError::FailToParse(anyhow!(
            "Failed to read file {} to buffer: {}",
            file.display(),
            errmsg
        ))
    })?;
    protobuf::Message::parse_from_bytes(data.as_ref()).map_err(|errmsg| {
        AconfigdError::FailToParse(anyhow!(
            "Failed to read file {} to buffer: {}",
            file.display(),
            errmsg
        ))
    })
}

/// Write pb to file
pub(crate) fn write_pb_to_file<T: protobuf::Message>(
    pb: &T,
    file: &Path,
) -> Result<(), AconfigdError> {
    let bytes = protobuf::Message::write_to_bytes(pb).map_err(|errmsg| {
        AconfigdError::FailToSerializePb(anyhow!(
            "Fail to serialize protobuf to bytes while writing to {}: {}",
            file.display(),
            errmsg
        ))
    })?;
    std::fs::write(file, bytes).map_err(|errmsg| {
        AconfigdError::FailToOverride(anyhow!(
            "Fail to write protobuf bytes to file {}: {}",
            file.display(),
            errmsg
        ))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use aconfigd_protos::ProtoLocalFlagOverrides;
    use tempfile::tempdir;

    fn get_file_perm_mode(file: &Path) -> u32 {
        let f = std::fs::File::open(&file).unwrap();
        let metadata = f.metadata().unwrap();
        metadata.permissions().mode() & 0o777
    }

    #[test]
    fn test_copy_file() {
        let tmp_dir = tempdir().unwrap();

        let package_map = tmp_dir.path().join("package.map");
        copy_file(&Path::new("./tests/data/package.map"), &package_map, 0o444).unwrap();
        assert_eq!(get_file_perm_mode(&package_map), 0o444);

        let flag_map = tmp_dir.path().join("flag.map");
        copy_file(&Path::new("./tests/data/flag.map"), &flag_map, 0o644).unwrap();
        assert_eq!(get_file_perm_mode(&flag_map), 0o644);
    }

    #[test]
    fn test_remove_file() {
        let tmp_dir = tempdir().unwrap();
        let package_map = tmp_dir.path().join("package.map");
        copy_file(&Path::new("./tests/data/package.map"), &package_map, 0o444).unwrap();
        assert!(remove_file(&package_map).is_ok());
        assert!(!package_map.exists());
    }

    #[test]
    fn test_set_file_permission() {
        let tmp_dir = tempdir().unwrap();
        let package_map = tmp_dir.path().join("package.map");
        copy_file(&Path::new("./tests/data/package.map"), &package_map, 0o644).unwrap();
        set_file_permission(&package_map, 0o444).unwrap();
        assert_eq!(get_file_perm_mode(&package_map), 0o444);
    }

    #[test]
    fn test_write_pb_to_file() {
        let tmp_dir = tempdir().unwrap();
        let test_pb = tmp_dir.path().join("test.pb");
        let pb = ProtoLocalFlagOverrides::new();
        write_pb_to_file(&pb, &test_pb).unwrap();
        assert!(test_pb.exists());
    }

    #[test]
    fn test_read_pb_from_file() {
        let tmp_dir = tempdir().unwrap();
        let test_pb = tmp_dir.path().join("test.pb");
        let pb = ProtoLocalFlagOverrides::new();
        write_pb_to_file(&pb, &test_pb).unwrap();
        let new_pb: ProtoLocalFlagOverrides = read_pb_from_file(&test_pb).unwrap();
        assert_eq!(new_pb.overrides.len(), 0);
    }
}
