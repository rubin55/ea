// SPDX-License-Identifier: AGPL-3.0-only
// Copyright © 2026 Rubin Simons

use std::path::Path;

use anyhow::Context;

fn read(p: &Path) -> anyhow::Result<Vec<u8>> {
  std::fs::read(p)
    .with_context(|| format!("failed to read {}", p.display()))
}

#[cfg(test)]
mod tests {
  use super::*;

  // TODO: Build a non-existing path using tempfile::tempdir()
  // TODO: Handle Result ok/err, consider Result::unwrap_err()
  #[test]
  fn non_existing_path() {
    read(Path::new("foo")).unwrap_err();
  }
}
