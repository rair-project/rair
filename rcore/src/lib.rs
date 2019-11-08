#![warn(clippy::cargo)]
#![allow(clippy::needless_return)]
/*
 * rcore: rair core library
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
extern crate app_dirs;
extern crate rio;
extern crate rtrees;
extern crate rustyline;
extern crate yansi;
mod core;
mod helper;
mod io;
mod loc;
pub use core::*;
pub use helper::*;
pub use io::*;
