/*
 *  Copyright (C) 2017  Ahmed Abd El Mawgood
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use libc::*;
pub struct RHashSeed {
    pub prefix:bool,
    pub buf: Vec<u8>,
}
#[repr(C)]
pub struct CRHashSeed {
    pub prefix: bool,
    pub buf: *const u8,
    pub len: usize,
}
impl RHashSeed {
    pub fn new() -> RHashSeed {
        RHashSeed{
            prefix:false,
            buf:Vec::new(),
        }
    }
}
#[repr(C)]
struct RMD5CTX {
    state:[u32;4],
    count:[u32;2],
    buffer:[u8;64],
}
#[repr(C)]
struct RSHACTX {
    h:[u32;5],
    w:[u32;80],
    lenw:i32,
    sizehi:u32,
    sizelo:u32,
}
#[repr(C)]
struct RSHA256CTX {
    state:[u32;8],
    bitcount:u64,
    buffer:[u8;64],
}
#[repr(C)]
struct RSHA512CTX {
    state:[u64;8],
    bitcount:[u64;2],
    buffer:[u8;128],
}
#[repr(C)]
pub struct RHash {
    md5:RMD5CTX,
    sha1:RSHACTX,
    sha256:RSHA256CTX,
    sha384:RSHA512CTX,
    sha512:RSHA512CTX,
    rst:bool,
    pub digest:[u8;128],
}
pub const R_HASH_ENTROPY:u64 = 1 << 11;
pub const R_HASH_ALL: u64 = 0x1FFFFFFFF;
pub const R_HASH_NBITS:u64 = 64;
pub const R_HASH_NONE:u64 = 0;
#[link(name = "r_hash")]
extern {
    pub fn r_hash_name_to_bits (STR: *const c_char) -> u64;
    pub fn r_hash_size(algo: u64) -> usize;
    pub fn r_hash_new(rst: bool, flags:u64) -> *const RHash;
    pub fn r_hash_name(bit:u64) -> *const c_char;
    pub fn r_hash_calculate(ctx:*const RHash, algobit:u64, buf: *const u8, len:usize) -> usize;
    pub fn r_hash_do_spice(ctx: *const RHash, algo:u64, loops:u64, seed:*const CRHashSeed);
    pub fn r_hash_entropy(buf: *const u8, len:usize) -> f64;
    pub fn r_hash_do_begin (ctx: *const RHash, i:u64);
    pub fn r_hash_do_end (ctx: *const RHash, i:u64);
}
