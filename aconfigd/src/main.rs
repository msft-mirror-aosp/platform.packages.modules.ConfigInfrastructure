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

//! `aconfigd-mainline` is a daemon binary that responsible for:
//! (1) initialize mainline storage files
//! (2) initialize and maintain a persistent socket based service

use clap::Parser;
use log::{error, info};

mod aconfigd_commands;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// start aconfigd socket.
    StartSocket,

    /// initialize mainline module storage files.
    Init,

    /// initialize bootstrap mainline module storage files.
    BootstrapInit,
}

fn main() {
    if !aconfig_new_storage_flags::enable_aconfig_storage_daemon()
        || !aconfig_new_storage_flags::enable_aconfigd_from_mainline()
    {
        info!("aconfigd_mainline is disabled, exiting");
        std::process::exit(0);
    }

    // SAFETY: nobody has taken ownership of the inherited FDs yet.
    // This needs to be called before logger initialization as logger setup will create a
    // file descriptor.
    unsafe {
        if let Err(errmsg) = rustutils::inherited_fd::init_once() {
            error!("failed to run init_once for inherited fds: {:?}.", errmsg);
            std::process::exit(1);
        }
    };

    // setup android logger, direct to logcat
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("aconfigd_mainline")
            .with_max_level(log::LevelFilter::Trace),
    );
    info!("starting aconfigd_mainline commands.");

    let cli = Cli::parse();
    let command_return = match cli.command {
        Command::StartSocket => aconfigd_commands::start_socket(),
        Command::Init => aconfigd_commands::init(),
        Command::BootstrapInit => aconfigd_commands::bootstrap_init(),
    };

    if let Err(errmsg) = command_return {
        error!("failed to run aconfigd command: {:?}.", errmsg);
        std::process::exit(1);
    }
}
