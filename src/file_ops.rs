use std::io::Result;
use std::ops::AddAssign;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use async_std::fs;
use async_std::path::PathBuf;
use async_std::stream::StreamExt;

use crate::copy_directions::SkipExt;

#[derive(Debug, Default)]
pub(crate) struct Summary {
    pub copied: usize,
    pub errors: usize,
    pub modified: usize,
    pub total: usize,
}

impl AddAssign for Summary {
    fn add_assign(&mut self, rhs: Self) {
        self.copied += rhs.copied;
        self.errors += rhs.errors;
        self.modified += rhs.modified;
        self.total += rhs.total;
    }
}

/// Recursively clone directory
///
/// Impl derived(copied) from
/// <https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust>
pub(crate) async fn copy<U, V>(
    from: U,
    to: V,
    skip_set: Option<SkipExt>,
    verbose: bool,
) -> Result<Summary>
where
    U: AsRef<Path> + std::hash::Hash + std::cmp::Eq,
    V: AsRef<Path>,
{
    assert_ne!(from.as_ref(), to.as_ref());
    assert!(from.as_ref().exists(), "Input directory not found");
    assert!(from.as_ref().is_dir(), "Input path is not a directory");
    let mut stack = vec![PathBuf::from(from.as_ref())];
    let empty = SkipExt::new();
    let skip_set = match skip_set {
        Some(s) => s,
        None => empty,
    };
    let mut summary = Summary::default();

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        if verbose {
            println!(
                "process: {:?} -> {:?}",
                working_path.to_str().unwrap(),
                output_root
                    .join(
                        working_path
                            .components()
                            .skip(input_root)
                            .collect::<PathBuf>()
                    )
                    .to_str()
                    .unwrap()
            );
        }

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).await.is_err() {
            if verbose {
                println!(" mkdir: {:?}", dest.to_str().unwrap());
            }
            fs::create_dir_all(&dest).await?;
        }

        let mut entries = fs::read_dir(working_path).await?;

        loop {
            let entry = entries.next().await;
            match entry {
                None => break,
                Some(entry) => {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_dir().await {
                        stack.push(path);
                    } else {
                        match path.file_name() {
                            Some(filename) => {
                                let ext = path.extension();
                                if let Some(ext) = ext {
                                    if skip_set.contains(ext.as_bytes()) {
                                        continue;
                                    }
                                }
                                let dest_path = dest.join(filename);
                                match dest_path.exists().await {
                                    true => {
                                        if dest_path.metadata().await?.modified()?
                                            < path.metadata().await?.modified()?
                                        {
                                            update_file(
                                                &path,
                                                &dest_path,
                                                "update",
                                                &mut summary.modified,
                                                &mut summary.errors,
                                                verbose,
                                            )
                                            .await;
                                        }
                                    }
                                    false => {
                                        update_file(
                                            &path,
                                            &dest_path,
                                            "copy",
                                            &mut summary.copied,
                                            &mut summary.errors,
                                            verbose,
                                        )
                                        .await;
                                    }
                                }
                            }
                            None => {
                                println!("failed: {:?}", path);
                                summary.errors += 1;
                            }
                        }
                        summary.total += 1;
                    }
                }
            }
        }
    }
    Ok(summary)
}

async fn update_file(
    from: &PathBuf,
    to: &PathBuf,
    id: &str,
    accumulator: &mut usize,
    err_accumulator: &mut usize,
    verbose: bool,
) {
    if verbose {
        println!(
            "  {}: {:?} -> {:?}",
            id,
            from.to_str().unwrap(),
            to.to_str().unwrap()
        );
    }
    match fs::copy(from, to).await {
        Ok(_) => {
            *accumulator += 1;
        }
        Err(_) => {
            *err_accumulator += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use async_std::task;

    use crate::file_ops::copy;
    use crate::test_utils::TestConfig;

    #[test]
    #[should_panic]
    fn copy_src_does_not_exist() {
        let c = TestConfig::new("desta", Some("ffsdfa"));
        task::spawn(copy(c.get_src(), c.get_dest(), None, true));
    }

    #[test]
    fn to_empty_dir() {
        let c = TestConfig::new("destb", None);
        let num_files = c.get_src().read_dir().unwrap().count();
        let handle = task::spawn(copy(c.get_src(), c.get_dest(), None, true));
        let copied = task::block_on(handle);
        assert_eq!(copied.unwrap().copied, num_files);
    }
}
