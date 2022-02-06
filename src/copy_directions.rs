use std::collections::HashMap;
use std::path::PathBuf;
use regex::Regex;

pub(crate) type FromPath = PathBuf;
pub(crate) type ToPath = PathBuf;
pub(crate) type SkipExt = Vec<Regex>;

pub(crate) type CopyDirections = HashMap<(FromPath, ToPath), SkipExt>;

#[derive(Debug)]
pub(crate) struct FileParseError {
    pub message: String,
}

pub(crate) fn from_string_list(data: Vec<Vec<u8>>) -> Result<CopyDirections, FileParseError> {
    let mut out = HashMap::new();
    let mut claimed_out_dirs: HashMap<PathBuf, PathBuf> = HashMap::new();
    for directions in data {
        let mut direction = directions.split(|v| *v == b':');
        let from_path: PathBuf;
        let to_path: PathBuf;
        let mut skip_exts = SkipExt::new();
        match direction.next() {
            Some(direction) => {
                from_path = PathBuf::from(&String::from_utf8(Vec::from(direction)).unwrap());
            }
            None => {
                return Err(FileParseError {
                    message: format!(
                        "Unable to parse `from_path` in string \"{}\"",
                        String::from_utf8(directions.clone()).unwrap()
                    ),
                });
            }
        }
        match direction.next() {
            Some(direction) => {
                to_path = PathBuf::from(&String::from_utf8(Vec::from(direction)).unwrap());
            }
            None => {
                return Err(FileParseError {
                    message: format!(
                        "Unable to parse `to_path` in string \"{}\"",
                        String::from_utf8(directions.clone()).unwrap()
                    ),
                });
            }
        }
        match claimed_out_dirs.contains_key(&to_path) {
            true => {
                return Err(FileParseError {
                    message: format!(
                        "`to_path` value \"{}\" is already in use for \"{}\"",
                        to_path.to_str().unwrap(),
                        claimed_out_dirs[&to_path].to_str().unwrap()
                    ),
                });
            }
            false => {
                claimed_out_dirs.insert(to_path.clone(), from_path.clone());
            }
        }
        if let Some(direction) = direction.next() {
            skip_exts = direction
                .split(|v| *v == b',')
                .filter_map(trim)
                .into_iter()
                .map(|v| Regex::new(std::str::from_utf8(&v).unwrap()).unwrap())
                .collect();
        }
        out.insert((from_path, to_path), skip_exts);
    }

    Ok(out)
}

fn trim(v: &[u8]) -> Option<Vec<u8>> {
    if !v.is_empty() {
        Some(v.to_vec())
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::copy_directions::from_string_list;

    static PRE: &str = "/home/user/pre";
    static POST: &str = "/home/user/post";

    fn to(val: &str) -> PathBuf {
        PathBuf::from(val)
    }

    #[test]
    fn simple() {
        let copy_directions = vec![format!("{}:{}", PRE, POST).as_bytes().to_vec()];
        let parsed_directions = from_string_list(copy_directions).unwrap();
        let pre = &to(PRE);
        let post = &to(POST);
        let id = &(pre.clone(), post.clone());
        assert!(parsed_directions.contains_key(id));
        assert!(parsed_directions.get(id).unwrap().is_empty());
    }
}
