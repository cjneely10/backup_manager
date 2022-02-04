use std::path::PathBuf;

lazy_static! {
    static ref SRC: PathBuf = PathBuf::from(PathBuf::from(file!()).parent().unwrap());
}

pub(crate) struct TestConfig {
    pub src: PathBuf,
    pub dest: PathBuf,
}

impl TestConfig {
    pub fn get_src(&self) -> PathBuf {
        self.src.clone()
    }

    pub fn get_dest(&self) -> PathBuf {
        self.dest.clone()
    }

    pub fn new(dest: &str, src: Option<&str>) -> TestConfig {
        let dest = PathBuf::from(dest);
        assert_ne!(dest, *SRC);
        match src {
            Some(s) => Self {
                src: PathBuf::from(s),
                dest,
            },
            None => Self {
                src: SRC.clone(),
                dest,
            },
        }
    }
}

impl Drop for TestConfig {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.get_dest()).unwrap();
    }
}
