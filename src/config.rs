use std::collections::{HashMap, HashSet};
use std::io::Result;
use std::path::PathBuf;

pub(crate) type FromPath = PathBuf;
pub(crate) type ToPath = PathBuf;
pub(crate) type SkipExt = HashSet<Vec<u8>>;

#[derive(Debug, Clone)]
pub(crate) struct CopyDirections(pub(crate) ToPath, pub(crate) SkipExt);

#[derive(Debug, Clone)]
pub(crate) struct Directions(pub(crate) HashMap<FromPath, CopyDirections>);

impl Directions {
    fn from_string_list(data: &[Vec<u8>]) -> Result<Self> {
        let mut out = HashMap::new();
        data.iter().for_each(|directions| {
            let mut direction = directions.split(|v| *v == b':');
            let from_path: PathBuf;
            let to_path: PathBuf;
            let mut skip_exts = SkipExt::new();
            match direction.next() {
                Some(direction) => {
                    from_path = PathBuf::from(&String::from_utf8(Vec::from(direction)).unwrap())
                }
                None => panic!(
                    "{}",
                    format!(
                        "Unable to parse `from_path` in string {}",
                        String::from_utf8(directions.clone()).unwrap()
                    )
                ),
            }
            match direction.next() {
                Some(direction) => {
                    to_path = PathBuf::from(&String::from_utf8(Vec::from(direction)).unwrap())
                }
                None => panic!(
                    "{}",
                    format!(
                        "Unable to parse `to_path` in string {}",
                        String::from_utf8(directions.clone()).unwrap()
                    )
                ),
            }
            if let Some(direction) = direction.next() {
                skip_exts = direction
                    .split(|v| *v == b',')
                    .filter_map(Self::trim)
                    .into_iter()
                    .collect();
            }

            out.insert(from_path, CopyDirections(to_path, skip_exts));
        });

        Ok(Self(out))
    }

    fn trim(v: &[u8]) -> Option<Vec<u8>> {
        let mut v: Vec<u8> = v
            .iter()
            .filter(|v| *v != &b' ' && *v != &b'.')
            .copied()
            .collect();
        if !v.is_empty() && v[0] == b'.' {
            v = v[1..].to_vec();
        }
        if !v.is_empty() {
            Some(v)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::config::Directions;
    use std::collections::HashSet;
    use std::path::PathBuf;
    static PRE: &str = "/home/user/pre";
    static POST: &str = "/home/user/post";
    static ARGS: &str = ".txt,.aln, .ttt";

    fn to(val: &str) -> PathBuf {
        PathBuf::from(val)
    }

    #[test]
    fn simple() {
        let copy_directions = vec![format!("{}:{}", PRE, POST).as_bytes().to_vec()];
        let parsed_directions = Directions::from_string_list(&copy_directions).unwrap();
        let pre = &to(PRE);
        let post = &to(POST);
        assert!(parsed_directions.0.contains_key(pre));
        assert_eq!(&parsed_directions.0.get(pre).unwrap().0, post);
        assert!(parsed_directions.0.get(pre).unwrap().1.is_empty());
    }

    #[test]
    fn with_args() {
        let copy_directions = vec![format!("{}:{}:{}", PRE, POST, ARGS).as_bytes().to_vec()];
        let parsed_directions = Directions::from_string_list(&copy_directions).unwrap();
        let pre = &to(PRE);
        let post = &to(POST);
        assert!(parsed_directions.0.contains_key(pre));
        assert_eq!(&parsed_directions.0.get(pre).unwrap().0, post);
        let args = parsed_directions.0.get(pre).unwrap().1.clone();
        let mut expected = HashSet::new();
        expected.insert("ttt".as_bytes().to_vec());
        expected.insert("txt".as_bytes().to_vec());
        expected.insert("aln".as_bytes().to_vec());
        assert!(expected.len() == args.len() && args.iter().all(|v| expected.contains(v)));
    }
}
