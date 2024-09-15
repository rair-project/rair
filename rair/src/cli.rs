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
        let mut perm = Default::default();
        for c in perms_str.to_lowercase().chars() {
            match c {
                'r' => perm |= IoMode::READ,
                'w' => perm |= IoMode::WRITE,
                'c' => perm |= IoMode::COW,
                _ => return Err(format!("Unknown Permission: `{}`", c)),
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
                Err("You cannot set Base address when opening a project".to_owned())
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
