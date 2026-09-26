// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Rubin Simons

mod codec;
mod format;
mod locator;
mod modules;
mod plan;
mod storage;

use std::path::{Path, PathBuf};

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
    profile: PathBuf,
    /// Compute the edits without writing them.
    #[arg(long)]
    dry_run: bool,
  },
}

fn cmd_get(key: &str) -> anyhow::Result<()> {
  todo!("get {key}")
}

fn cmd_set(key: &str, value: &str) -> anyhow::Result<()> {
  todo!("set {key} = {value}")
}

fn cmd_dump() -> anyhow::Result<()> {
  todo!("dump")
}

fn cmd_describe() -> anyhow::Result<()> {
  todo!("describe")
}

fn cmd_apply(profile: &Path, dry_run: bool) -> anyhow::Result<()> {
  todo!("apply {profile:?} dry_run={dry_run}")
}

fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  match cli.command {
    Command::Get { key } => cmd_get(&key),
    Command::Set { key, value } => cmd_set(&key, &value),
    Command::Dump => cmd_dump(),
    Command::Describe => cmd_describe(),
    Command::Apply { profile, dry_run } => cmd_apply(&profile, dry_run),
  }
}
