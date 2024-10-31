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
use std::panic;

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
    // setup android logger, direct to logcat
    android_logger::init_once(
        android_logger::Config::default()
            .with_tag("aconfigd_mainline")
            .with_max_level(log::LevelFilter::Trace),
    );
    info!("starting aconfigd_mainline commands.");

    // redirect panic messages to logcat.
    panic::set_hook(Box::new(|panic_info| {
        error!("{}", panic_info);
    }));

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
