//! Native path replacements retain the original file before changing its shape.

use crate::test_support::create_private_fifo;
use std::{fs, os::unix::fs::symlink, path::Path};

pub(super) enum Replacement<'a> {
    File(&'a [u8]),
    Symlink,
    Directory,
    Missing,
    Fifo,
}

impl Replacement<'_> {
    pub(super) fn apply(self, path: &Path, retained: &Path) {
        fs::rename(path, retained).unwrap();
        match self {
            Self::File(bytes) => fs::write(path, bytes).unwrap(),
            Self::Symlink => symlink(retained, path).unwrap(),
            Self::Directory => fs::create_dir(path).unwrap(),
            Self::Missing => {}
            Self::Fifo => create_private_fifo(path),
        }
    }
}
