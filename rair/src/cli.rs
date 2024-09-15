use clap::Parser;
use rair_core::str_to_num;
use rair_io::IoMode;

#[derive(Parser, Debug)]
#[command(version)]
#[command(about = "reverse engineering framework")]
struct ArgsInner {
    /// File permision: Permission can be R, C, or RW case insensitive
    #[arg(short = 'p', long = "perm")]
    #[arg(value_name = "rwc")]
    pub perm: Option<String>,

    /// Physical Base address, Default
    #[arg(short = 'b', long = "base")]
    #[arg(value_name = "num")]
    pub base: Option<String>,

    /// Project to be opened
    #[arg(short = 'P', long = "proj")]
    #[arg(value_name = "/path/to/project")]
    pub proj: Option<String>,

    /// Binary file to be loaded
    pub file: Option<String>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Args {
    Proj(String),
    File {
        uri: String,
        base: u64,
        perms: IoMode,
    },
}
impl Args {
    fn parse_perm(perms_str: &str) -> Result<IoMode, String> {
        let mut perm = IoMode::default();
        for c in perms_str.chars() {
            match c.to_ascii_lowercase() {
                'r' => perm |= IoMode::READ,
                'w' => perm |= IoMode::WRITE,
                'c' => perm |= IoMode::COW,
                _ => return Err(format!("Unknown Permission: `{c}`")),
            }
        }
        Ok(perm)
    }
    /// parse command line arguments
    pub fn parse() -> Result<Self, String> {
        ArgsInner::parse().try_into()
    }
}

impl TryFrom<ArgsInner> for Args {
    type Error = String;

    fn try_from(ai: ArgsInner) -> Result<Self, Self::Error> {
        match (ai.proj, ai.file, ai.base, ai.perm) {
            (Some(_), Some(_), _, _) | (None, None, _, _) => {
                Err("You must open either a binary file or Project file, but not both".to_owned())
            }
            (Some(_), _, Some(_), _) => {
                Err("You cannot set base address when opening a project".to_owned())
            }
            (Some(_), _, _, Some(_)) => {
                Err("You cannot set permissions when opening a project".to_owned())
            }
            (Some(proj), None, None, None) => Ok(Args::Proj(proj)),
            (None, Some(uri), base, perms) => {
                let perms = Self::parse_perm(&perms.unwrap_or("r".to_owned()))?;
                let base =
                    str_to_num(&base.unwrap_or("0x0".to_owned())).map_err(|e| e.to_string())?;
                Ok(Args::File { uri, base, perms })
            }
        }
    }
}

#[cfg(test)]
mod cli_tests {
    use rair_io::IoMode;

    use super::{Args, ArgsInner};

    #[test]
    fn parse_perm_small() {
        let p1 = Args::parse_perm("c").unwrap();
        assert_eq!(p1, IoMode::COW);
        let p1 = Args::parse_perm("r").unwrap();
        assert_eq!(p1, IoMode::READ);
        let p1 = Args::parse_perm("w").unwrap();
        assert_eq!(p1, IoMode::WRITE);
    }

    #[test]
    fn parse_perm_capital() {
        let p1 = Args::parse_perm("C").unwrap();
        assert_eq!(p1, IoMode::COW);
        let p1 = Args::parse_perm("R").unwrap();
        assert_eq!(p1, IoMode::READ);
        let p1 = Args::parse_perm("W").unwrap();
        assert_eq!(p1, IoMode::WRITE);
    }

    #[test]
    fn parse_perm_mixed() {
        let p1 = Args::parse_perm("rW").unwrap();
        assert_eq!(p1, IoMode::READ | IoMode::WRITE);
    }

    #[test]
    fn parse_perm_bad() {
        let err = Args::parse_perm("rWX").err().unwrap();
        assert_eq!(err, "Unknown Permission: `X`");
    }

    #[test]
    fn proj_test() {
        let ai = ArgsInner {
            perm: None,
            base: None,
            proj: Some("hello".to_owned()),
            file: None,
        };
        let args: Args = ai.try_into().unwrap();
        assert_eq!(args, Args::Proj("hello".to_owned()));
    }
    #[test]
    fn file_test() {
        let ai = ArgsInner {
            perm: None,
            base: None,
            proj: None,
            file: Some("hello".to_owned()),
        };
        let args: Args = ai.try_into().unwrap();
        assert_eq!(
            args,
            Args::File {
                uri: "hello".to_owned(),
                base: 0,
                perms: IoMode::READ
            }
        );
    }
    #[test]
    fn file_with_attributes_test() {
        let ai = ArgsInner {
            perm: Some("c".to_owned()),
            base: Some("0x1000".to_owned()),
            proj: None,
            file: Some("hello".to_owned()),
        };
        let args: Args = ai.try_into().unwrap();
        assert_eq!(
            args,
            Args::File {
                uri: "hello".to_owned(),
                base: 0x1000,
                perms: IoMode::COW
            }
        );
    }

    #[test]
    fn proj_file() {
        let ai = ArgsInner {
            perm: None,
            base: None,
            proj: Some("hello".to_owned()),
            file: Some("hello".to_owned()),
        };
        let err: Result<Args, _> = ai.try_into();
        let err = err.err().unwrap();
        assert_eq!(
            err,
            "You must open either a binary file or Project file, but not both"
        );
    }

    #[test]
    fn proj_base() {
        let ai = ArgsInner {
            perm: None,
            base: Some("0x1000".to_owned()),
            proj: Some("hello".to_owned()),
            file: None,
        };
        let err: Result<Args, _> = ai.try_into();
        let err = err.err().unwrap();
        assert_eq!(err, "You cannot set base address when opening a project");
    }
    #[test]
    fn proj_perm() {
        let ai = ArgsInner {
            perm: Some("c".to_owned()),
            base: None,
            proj: Some("hello".to_owned()),
            file: None,
        };
        let err: Result<Args, _> = ai.try_into();
        let err = err.err().unwrap();
        assert_eq!(err, "You cannot set permissions when opening a project");
    }
}
