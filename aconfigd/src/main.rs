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

use aconfigd_mainline::AconfigdError;
use clap::Parser;

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

fn main() -> Result<(), AconfigdError> {
    let cli = Cli::parse();
    match cli.command {
        Command::StartSocket => aconfigd_commands::start_socket()?,
        Command::Init => aconfigd_commands::init()?,
        Command::BootstrapInit => aconfigd_commands::bootstrap_init()?,
    };
    Ok(())
}
