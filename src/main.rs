// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Rubin Simons

use clap::{Parser, Subcommand};

/// Everything anything. Enables uniform access to "entities" which are k/v
/// pairs in various storages. Storages can be tool based or file based.
/// For file based storage, it is file format aware.
#[derive(Parser)]
#[command(name = "ea", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read one value.
    Get { key: String },
    /// Write one value.
    Set { key: String, value: String },
    /// Read every value this host exposes, as a profile.
    Dump,
    /// List the keys that exist, installed or not.
    Describe,
    /// Set many values at once from a profile.
    Apply {
        profile: std::path::PathBuf,
        /// Compute the edits without writing them.
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Get { key } => todo!("get {key}"),
        Command::Set { key, value } => todo!("set {key} = {value}"),
        Command::Dump => todo!("dump"),
        Command::Describe => todo!("describe"),
        Command::Apply { profile, dry_run } => {
            todo!("apply {profile:?} dry_run={dry_run}")
        }
    }
}
