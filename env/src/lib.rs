//! rair environment variables library.
extern crate alloc;
mod environment;
mod err;
mod metadata;

pub use environment::*;
pub use err::*;
pub use metadata::*;
