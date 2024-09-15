//! error handling for renv.

use alloc::fmt;

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum EnvErr {
    NotFound,
    DifferentType,
    CbFailed,
    AlreadyExist,
}

impl fmt::Display for EnvErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvErr::NotFound => write!(f, "Environment variable not found."),
            EnvErr::DifferentType => write!(f, "Environment variable has different type."),
            EnvErr::CbFailed => write!(f, "Call back failed."),
            EnvErr::AlreadyExist => write!(f, "Environment variable already exist."),
        }
    }
}
