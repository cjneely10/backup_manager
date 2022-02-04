use std::io::Result;

use async_std::task::{block_on, spawn};

use crate::copy_directions::CopyDirections;
use crate::file_ops::copy;

async fn run(directions: CopyDirections) -> Result<usize> {
    let mut handles = Vec::new();
    let mut total = 0;
    for (to_path, cfg) in directions {
        let skip_set = match cfg.1.len() {
            0 => None,
            _ => Some(cfg.1),
        };
        handles.push(spawn(copy(to_path, cfg.0, skip_set)));
    }
    for handle in handles {
        total += handle.await.unwrap();
    }
    Ok(total)
}

/// Initialize copier pool for each direction tuple and begin copy process
pub(crate) fn execute(directions: CopyDirections) -> Result<usize> {
    block_on(run(directions))
}

#[cfg(test)]
mod test {
    use crate::test_utils::TestConfig;
    use crate::{execute, from_string_list};

    #[test]
    fn copy_all() {
        let c = TestConfig::new("destc", None);
        let num_files = c.src.read_dir().unwrap().count();
        let copied = execute(
            from_string_list(vec![format!(
                "{}:{}",
                c.src.to_str().unwrap(),
                c.dest.to_str().unwrap()
            )
            .as_bytes()
            .to_vec()])
            .unwrap(),
        );
        assert_eq!(copied.unwrap(), num_files);
    }
}
