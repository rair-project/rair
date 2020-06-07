/*
 * dummy.rs: RIO dummy plugin: It does nothing at all, usually it is used when reopening files.
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

use plugin::*;
use utils::*;
pub struct Dummy {}

impl RIOPluginOperations for Dummy {
    fn read(&mut self, _raddr: usize, _buffer: &mut [u8]) -> Result<(), IoError> {
        Ok(())
    }
    fn write(&mut self, _raddr: usize, _buffer: &[u8]) -> Result<(), IoError> {
        Ok(())
    }
}
