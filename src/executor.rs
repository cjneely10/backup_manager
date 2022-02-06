use async_std::task::{block_on, spawn};

use crate::copy_directions::CopyDirections;
use crate::file_ops::copy;
use crate::summary::Summary;

/// Initialize copier pool for each direction tuple and begin copy process
pub(crate) fn execute(directions: CopyDirections, verbose: bool) -> Summary {
    block_on(run(directions, verbose))
}

async fn run(directions: CopyDirections, verbose: bool) -> Summary {
    directions.iter().for_each(|(paths, exclusions)| {
        print!(
            "{} -> {}",
            paths.0.to_str().unwrap(),
            paths.1.to_str().unwrap()
        );
        if !exclusions.is_empty() {
            println!(", excluding {:?}", exclusions);
        } else {
            println!();
        }
    });
    let mut handles = Vec::new();
    let mut total = Summary::default();
    for (paths, cfg) in directions {
        let skip_set = match cfg.len() {
            0 => None,
            _ => Some(cfg),
        };
        handles.push(spawn(copy(paths.0, paths.1, skip_set, verbose)));
    }
    for handle in handles {
        match handle.await {
            Ok(summary) => {
                total += summary;
            }
            Err(e) => {
                eprintln!("{}", e)
            }
        }
    }
    total
}

#[cfg(test)]
mod test {
    use crate::test_utils::test_config::TestConfig;
    use crate::{execute, from_string_list};

    #[test]
    fn copy_all() {
        let c = TestConfig::new("destea", None);
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
            true,
        );
        assert_eq!(copied.new, num_files);
    }

    #[test]
    fn copy_excluded() {
        let c = TestConfig::new("desteb", None);
        let copied = execute(
            from_string_list(vec![format!(
                "{}:{}:.rs",
                c.src.to_str().unwrap(),
                c.dest.to_str().unwrap()
            )
            .as_bytes()
            .to_vec()])
            .unwrap(),
            true,
        );
        assert_eq!(copied.new, 0);
    }
}
