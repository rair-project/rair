//!various trees impelementation for rair project

#[cfg(feature = "serialize")]
extern crate serde;

/// Approximate String search data structure.
pub mod bktree;
/// Interval search tree implementation.
pub mod ist;

/// Left-Leaning Red Black tree implementation built with augmentation in mind.
pub mod rbtree;
