#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![warn(future_incompatible, nonstandard_style, warnings, rust_2018_idioms, unused, rust_2018_idioms, missing_docs)]

//!various trees impelementation for rair project

/*
 * rtrees: rair trees library impelementation
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */
#[cfg(test)]
extern crate serde_json;

#[cfg(feature = "serialize")]
extern crate serde;

/// Approximate String search data structure.
pub mod bktree;
/// Interval search tree implementation.
pub mod ist;

/// Left-Leaning Red Black tree implementation built with augmentation in mind.
pub mod rbtree;
