//! keep track of whatever files needs to be taken care of (config, hist ..etc).

use directories::ProjectDirs;
use std::path::PathBuf;

fn project_dir() -> ProjectDirs {
    ProjectDirs::from("com", "Rair", "rair").unwrap()
}

pub fn hist_file() -> PathBuf {
    project_dir().data_dir().join("history")
}

#[cfg(test)]
mod test_files {
    use super::*;
    #[test]
    fn history() {
        let hist = hist_file();
        assert!(hist.ends_with("history"));
    }
}
