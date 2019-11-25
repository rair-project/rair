#![warn(clippy::cargo)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::needless_return)]
/*
 * rio: rair IO library impelementation
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

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate nom;

extern crate base64;
extern crate memmap;
extern crate rtrees;
#[cfg(test)]
extern crate test_file;

mod desc;
mod descquery;
mod io;
mod mapsquery;
mod plugin;
mod plugins;
mod utils;
pub use desc::*;
pub use io::*;
pub use mapsquery::*;
pub use plugin::*;
pub use utils::*;
