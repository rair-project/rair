#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
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
pub mod bktree;
pub mod ist;
pub mod rbtree;
