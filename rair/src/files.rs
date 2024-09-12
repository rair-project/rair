//! keep track of whatever files needs to be taken care of (config, hist ..etc).

use app_dirs::*;
use std::path::PathBuf;

const APPINFO: AppInfo = AppInfo {
    name: "rair",
    author: "RairDevs",
};

pub fn hist_file() -> PathBuf {
    let mut history = app_dir(AppDataType::UserData, &APPINFO, "/").unwrap();
    history.push("history");
    return history;
}

#[cfg(test)]

mod test_files {
    use super::*;
    #[test]
    fn test_history() {
        let hist = hist_file();
        assert!(hist.ends_with("history"));
    }
}
