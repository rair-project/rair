//! commands handling data writing to files.
#[cfg(test)]
mod tests;
mod writehex;
mod writetofile;

pub use writehex::WriteHex;
pub use writetofile::WriteToFile;
