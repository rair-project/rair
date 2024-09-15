//! Utility data structures for managing RIO.

use alloc::fmt;
use bitflags::bitflags;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::io;

bitflags! {
    /// Set the mode for opening files.
    #[derive(Default, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
    pub struct IoMode: u64 {
    /// Open File in read mode.
    const WRITE = 2;
    /// Open file in write mode.
    const READ = 4;
    /// Open file in Copy-On-Write mode.
    const COW = 8;
    }
}

impl fmt::Display for IoMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.iter_names().map(|(name, _)| name).join(" | ");
        f.write_str(&s)
    }
}

/// Errors resultion from operations on [RIO]
#[derive(Debug)]
#[non_exhaustive]
pub enum IoError {
    /// Reading or writing to an invalid address.
    AddressNotFound,
    /// Memory addresses gets mapped in way that makes them overlap
    AddressesOverlapError,
    /// There is no sutiable IO plugin for loading the given file encoding
    IoPluginNotFoundError,
    /// Doing operationg on file handles that doesn't exist
    HndlNotFoundError,
    /// Too many files are opened.
    TooManyFilesError,
    /// Custom error message.
    Custom(String),
    /// Error that is originating from [`std::io`]
    Parse(io::Error),
}
impl PartialEq for IoError {
    fn eq(&self, other: &IoError) -> bool {
        match self {
            IoError::AddressNotFound => {
                if let IoError::AddressNotFound = other {
                    return true;
                }
            }
            IoError::AddressesOverlapError => {
                if let IoError::AddressesOverlapError = other {
                    return true;
                }
            }
            IoError::IoPluginNotFoundError => {
                if let IoError::IoPluginNotFoundError = other {
                    return true;
                }
            }
            IoError::TooManyFilesError => {
                if let IoError::TooManyFilesError = other {
                    return true;
                }
            }
            IoError::HndlNotFoundError => {
                if let IoError::HndlNotFoundError = other {
                    return true;
                }
            }
            IoError::Custom(s) => {
                if let IoError::Custom(s2) = other {
                    return s == s2;
                }
            }
            IoError::Parse(_) => {
                if let IoError::Parse(_) = other {
                    return true;
                }
            }
        }
        false
    }
}
impl fmt::Display for IoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IoError::AddressNotFound => write!(f, "Cannot resolve address."),
            IoError::AddressesOverlapError => write!(f, "Phyiscal addresses overlap."),
            IoError::IoPluginNotFoundError => write!(f, "Can not find Suitable IO Plugin."),
            IoError::TooManyFilesError => write!(f, "You have too many open files."),
            IoError::HndlNotFoundError => write!(f, "Handle Does not exist."),
            IoError::Custom(s) => write!(f, "{s}."),
            IoError::Parse(e) => e.fmt(f),
        }
    }
}

impl From<io::Error> for IoError {
    fn from(err: io::Error) -> IoError {
        IoError::Parse(err)
    }
}
