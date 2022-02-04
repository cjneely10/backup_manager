use async_std::path::PathBuf;
use async_std::task::{block_on, spawn};
use std::io::Result;

use crate::copy_directions::CopyDirections;
use crate::file_ops::copy;

pub(crate) type PathBufTuple = (PathBuf, PathBuf);

async fn run(directions: CopyDirections) -> Result<()> {
    let mut handles = Vec::new();
    for (to_path, cfg) in directions {
        let skip_set = match cfg.1.len() {
            0 => None,
            _ => Some(cfg.1),
        };
        handles.push(spawn(copy(to_path, cfg.0, skip_set)));
    }
    Ok(())
}

/// Initialize copier pool for each direction tuple and begin copy process
pub(crate) fn execute(directions: CopyDirections) -> Result<()> {
    block_on(run(directions))
}
