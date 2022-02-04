use std::io::Result;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use async_std::fs;
use async_std::path::PathBuf;
use async_std::stream::StreamExt;

use crate::copy_directions::SkipExt;

/// Recursively clone directory
///
/// Impl derived(copied) from
/// <https://stackoverflow.com/questions/26958489/how-to-copy-a-folder-recursively-in-rust>
pub(crate) async fn copy<U, V>(from: U, to: V, skip_set: Option<SkipExt>) -> Result<usize>
where
    U: AsRef<Path> + std::hash::Hash + std::cmp::Eq,
    V: AsRef<Path>,
{
    assert!(from.as_ref().exists(), "Input directory not found");
    assert!(from.as_ref().is_dir(), "Input path is not a directory");
    let mut file_count = 0;
    let mut stack = vec![PathBuf::from(from.as_ref())];
    let empty = SkipExt::new();
    let skip_set = match skip_set {
        Some(s) => s,
        None => empty,
    };

    let output_root = PathBuf::from(to.as_ref());
    let input_root = PathBuf::from(from.as_ref()).components().count();

    while let Some(working_path) = stack.pop() {
        println!("process: {:?}", &working_path);

        // Generate a relative path
        let src: PathBuf = working_path.components().skip(input_root).collect();

        // Create a destination if missing
        let dest = if src.components().count() == 0 {
            output_root.clone()
        } else {
            output_root.join(&src)
        };
        if fs::metadata(&dest).await.is_err() {
            println!(" mkdir: {:?}", dest);
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
                                let ext = path.extension().unwrap().as_bytes();
                                if skip_set.contains(ext) {
                                    continue;
                                }
                                let dest_path = dest.join(filename);
                                println!("  copy: {:?} -> {:?}", &path, &dest_path);
                                file_count += 1;
                                // TODO: Copy new or modified files and filter files not present
                                fs::copy(&path, &dest_path).await?;
                            }
                            None => {
                                println!("failed: {:?}", path);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(file_count)
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use async_std::task;

    use crate::file_ops::copy;

    struct Cleanup;

    impl Drop for Cleanup {
        fn drop(&mut self) {
            std::fs::remove_dir_all("dest").unwrap();
        }
    }

    #[test]
    #[should_panic]
    fn copy_src_does_not_exist() {
        let _c = Cleanup;
        let src = PathBuf::from("s");
        let dest = PathBuf::from("dest");
        let _ = task::spawn(copy(src, dest, None));
    }

    #[test]
    fn to_empty_dir() {
        let _c = Cleanup;
        let src = PathBuf::from("src");
        let num_files = src.read_dir().unwrap().count();
        let dest = PathBuf::from("dest");
        let handle = task::spawn(copy(src, dest, None));
        let copied = task::block_on(handle);
        assert_eq!(copied.unwrap(), num_files);
    }
}
