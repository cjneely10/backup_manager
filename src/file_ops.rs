use std::io::Result;
use std::path::Path;

use async_std::fs;
use async_std::path::PathBuf;
use async_std::stream::StreamExt;
use async_std::task::{spawn_local, yield_now, JoinHandle};

use crate::backup_instruction_parser::SkipPatterns;
use crate::summary::Summary;

pub(crate) const COPY: &str = "copy";
pub(crate) const MODIFY: &str = "update";

/// Recursively clone directory
///
/// Impl derived(copied) from
/// <https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust>
pub(crate) async fn copy<U, V>(
    from: U,
    to: V,
    skip_set: Option<SkipPatterns>,
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
    let empty = SkipPatterns::new();
    let skip_set = match skip_set {
        Some(s) => s,
        None => empty,
    };
    let mut summary = Summary::default();

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    let mut handles = Vec::new();

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
                    let path = entry.unwrap().path();
                    if path.is_dir().await {
                        stack.push(path);
                    } else {
                        process_file(&mut summary, &path, &dest, &skip_set, verbose, &mut handles)
                            .await?;
                    }
                }
            }
        }
    }
    Ok(summary.summarize(handles).await)
}

async fn process_file(
    summary: &mut Summary,
    path: &PathBuf,
    dest: &PathBuf,
    skip_set: &SkipPatterns,
    verbose: bool,
    handles: &mut Vec<JoinHandle<(&str, bool)>>,
) -> Result<()> {
    match path.file_name() {
        Some(filename) => {
            if !skip_set.is_empty()
                && skip_set
                    .iter()
                    .any(|re| re.is_match(path.to_str().unwrap()))
            {
                return Ok(());
            }
            let dest_path = dest.join(filename);
            match dest_path.exists().await {
                true => {
                    if dest_path.metadata().await?.modified()?
                        < path.metadata().await?.modified()?
                    {
                        update_file(path, &dest_path, MODIFY, verbose, handles).await;
                    } else {
                        summary.existing += 1;
                    }
                }
                false => {
                    update_file(path, &dest_path, COPY, verbose, handles).await;
                }
            }
        }
        None => {
            eprintln!("failed: {:?}", path);
            summary.errors += 1;
        }
    }
    summary.total += 1;
    Ok(())
}

async fn file_copy_runner(from: PathBuf, to: PathBuf, id: &'static str) -> (&'static str, bool) {
    match fs::copy(from, to).await {
        Ok(_) => (id, true),
        Err(e) => {
            eprintln!("{}", e);
            (id, false)
        }
    }
}

async fn update_file(
    from: &PathBuf,
    to: &PathBuf,
    id: &'static str,
    verbose: bool,
    handles: &mut Vec<JoinHandle<(&str, bool)>>,
) {
    if verbose {
        println!(
            "  {}: {:?} -> {:?}",
            id,
            from.to_str().unwrap(),
            to.to_str().unwrap()
        );
    }
    handles.push(spawn_local(file_copy_runner(from.clone(), to.clone(), id)));
    yield_now().await;
}

#[cfg(test)]
mod test {
    use async_std::task;
    use async_std::task::block_on;

    use crate::file_ops::copy;
    use crate::test_utils::test_config::TestConfig;

    #[test]
    #[should_panic]
    fn copy_src_does_not_exist() {
        let c = TestConfig::new("destfa", Some("ffsdfa"));
        block_on(task::spawn(copy(c.get_src(), c.get_dest(), None, true))).unwrap();
    }

    #[test]
    fn to_empty_dir() {
        let c = TestConfig::new("destfb", None);
        let num_files = c.get_src().read_dir().unwrap().count();
        let handle = task::spawn(copy(c.get_src(), c.get_dest(), None, true));
        let copied = task::block_on(handle).unwrap();
        assert_eq!(copied.new, num_files);
        let handle = task::spawn(copy(c.get_src(), c.get_dest(), None, true));
        let copied = task::block_on(handle).unwrap();
        assert_eq!(copied.new, 0);
        assert_eq!(copied.existing, num_files);
    }
}
