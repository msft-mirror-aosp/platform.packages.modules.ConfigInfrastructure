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

use aconfigd_mainline::aconfigd::Aconfigd;
use aconfigd_mainline::AconfigdError;
use aconfigd_protos::ProtoStorageReturnMessage;
use anyhow::anyhow;
use log::{log, Level};
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;

const ACONFIGD_SOCKET: &str = "/dev/socket/aconfigd";
const ACONFIGD_ROOT_DIR: &str = "/metadata/aconfig";
const STORAGE_RECORDS: &str = "/metadata/aconfig/storage_records.pb";

/// Handle socket request from a unix stream
pub fn handle_socket_request_from_stream(
    aconfigd: &mut Aconfigd,
    stream: &mut UnixStream,
) -> Result<(), AconfigdError> {
    let mut request = String::new();
    stream.read_to_string(&mut request).map_err(|errmsg| {
        AconfigdError::FailToReadFromSocket(anyhow!(
            "Fail to read from socket unix stream: {}",
            errmsg
        ))
    })?;

    let request = &protobuf::Message::parse_from_bytes(request.as_bytes()).map_err(|errmsg| {
        AconfigdError::FailToParse(anyhow!("Failed to parse into protobuf to buffer: {}", errmsg))
    })?;

    let return_pb = match aconfigd.handle_socket_request(request) {
        Ok(return_msg) => return_msg,
        Err(errmsg) => {
            log!(Level::Error, "failed to handle socket request: {}", errmsg);
            let mut return_msg = ProtoStorageReturnMessage::new();
            return_msg.set_error_message(format!("failed to handle socket request: {}", errmsg));
            return_msg
        }
    };

    let bytes = protobuf::Message::write_to_bytes(&return_pb).map_err(|errmsg| {
        AconfigdError::FailToSeralizePb(anyhow!("Fail to serialize protobuf to bytes: {}", errmsg))
    })?;

    stream.write(&bytes).map_err(|errmsg| {
        AconfigdError::FailToWriteToSocket(anyhow!("Fail to write to a socket: {}", errmsg))
    })?;

    Ok(())
}

/// start aconfigd socket service
pub fn start_socket() -> Result<(), AconfigdError> {
    let mut aconfigd = Aconfigd::new(Path::new(ACONFIGD_ROOT_DIR), Path::new(STORAGE_RECORDS));
    let listener = UnixListener::bind(Path::new(ACONFIGD_SOCKET)).map_err(|errmsg| {
        AconfigdError::FailToBindSocket(anyhow!("Fail to bind to /dev/socket/aconfigd: {}", errmsg))
    })?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(errmsg) = handle_socket_request_from_stream(&mut aconfigd, &mut stream) {
                    log!(Level::Error, "failed to handle socket request: {}", errmsg);
                }
            }
            Err(errmsg) => {
                log!(Level::Error, "failed to get unix stream: {}", errmsg);
            }
        }
    }
    Ok(())
}

/// initialize mainline module storage files
pub fn init() -> Result<(), AconfigdError> {
    let mut aconfigd = Aconfigd::new(Path::new(ACONFIGD_ROOT_DIR), Path::new(STORAGE_RECORDS));
    aconfigd.initialize_mainline_storage()
}

/// initialize bootstrapped mainline module storage files
pub fn bootstrap_init() -> Result<(), AconfigdError> {
    Ok(())
}
