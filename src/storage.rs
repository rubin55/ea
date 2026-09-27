// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Rubin Simons

use std::path::Path;

use anyhow::Context;

fn read(p: &Path) -> anyhow::Result<Vec<u8>> {
  std::fs::read(p).with_context(|| format!("failed to read {}", p.display()))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn non_existing_path() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("some.file");
    let err = read(&path).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains(&path.display().to_string()));
  }
}
