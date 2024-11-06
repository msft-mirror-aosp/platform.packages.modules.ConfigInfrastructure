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
use anyhow::anyhow;
use log::{error, info};
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixListener;
use std::path::Path;

const ACONFIGD_SOCKET: &str = "aconfigd_mainline";
const ACONFIGD_ROOT_DIR: &str = "/metadata/aconfig";
const STORAGE_RECORDS: &str = "/metadata/aconfig/storage_records.pb";

/// start aconfigd socket service
#[cfg(not(feature = "cargo"))]
pub fn start_socket() -> Result<(), AconfigdError> {
    // SAFETY: nobody has taken ownership of the inherited FDs yet.
    unsafe {
        rustutils::inherited_fd::init_once().map_err(|errmsg| {
            AconfigdError::FailToBindSocket(anyhow!(
                "fail to init once to set CLOEXEC flag: {}",
                errmsg
            ))
        })
    };

    let fd = rustutils::sockets::android_get_control_socket(ACONFIGD_SOCKET).map_err(|errmsg| {
        AconfigdError::FailToBindSocket(anyhow!(
            "fail to get control socket {}'s owned file descriptor: {:?}",
            ACONFIGD_SOCKET,
            errmsg
        ))
    })?;

    // SAFETY: Safe because this doesn't modify any memory and we check the return value.
    let ret = unsafe { libc::listen(fd.as_raw_fd(), 8) };
    if ret < 0 {
        let listen_err = std::io::Error::last_os_error();
        return Err(AconfigdError::FailToBindSocket(anyhow!(
            "fail to listen to socket: {:?}",
            listen_err
        )));
    }

    let listener = UnixListener::from(fd);

    let mut aconfigd = Aconfigd::new(Path::new(ACONFIGD_ROOT_DIR), Path::new(STORAGE_RECORDS));

    loop {
        info!("wait for a new client connection through socket.");
        match listener.accept() {
            Ok((mut stream, _)) => {
                if let Err(errmsg) = aconfigd.handle_socket_request_from_stream(&mut stream) {
                    error!("failed to handle socket request: {:?}", errmsg);
                }
            }
            Err(errmsg) => {
                error!("accept function failed: {:?}", errmsg);
            }
        }
    }
}

#[cfg(feature = "cargo")]
pub fn start_socket() -> Result<(), AconfigdError> {
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
