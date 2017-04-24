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
use std::ffi::{CStr, CString};
#[link(name = "r_crypto")]
extern "C" {
    fn r_crypto_name(bit: u64) -> *const c_char;
    fn r_crypto_new() -> *const c_void;
    fn r_crypto_use(crypto: *const c_void, algo: *const c_char) -> bool;
    fn r_crypto_set_key(crypto: *const c_void,
                        key: *const u8,
                        len: usize,
                        mode: i32,
                        is_decryption: bool)
                        -> bool;
    fn r_crypto_set_iv(crypto: *const c_void, iv: *const u8, ivlen: usize) -> bool;
    fn r_crypto_update(crypto: *const c_void, buf: *const u8, buflen: usize);
    fn r_crypto_final(crypto: *const c_void, buf: *const u8, buflen: usize);
    fn r_crypto_get_output(crypto: *const c_void, size: *const usize) -> *mut u8;
}

pub fn get_output(crypto: *const c_void) -> Vec<u8> {
    let x: usize = 0;
    let y: *mut u8 = unsafe { r_crypto_get_output(crypto, &x) };
    unsafe { Vec::from_raw_parts(y, x, x) }
}
pub fn name(bits: u64) -> String {
    let name = unsafe { r_crypto_name(bits) };
    unsafe { CStr::from_ptr(name) }
        .to_string_lossy()
        .into_owned()
}
pub fn use_algo(crypto: *const c_void, algo: &str) -> bool {
    let cstr = CString::new(algo).unwrap();
    unsafe { r_crypto_use(crypto, cstr.as_ptr()) }
}
pub fn new() -> *const c_void {
    unsafe { r_crypto_new() }
}
pub fn set_key(crypto: *const c_void, key: &[u8], mode: i32, is_decryption: bool) -> bool {
    unsafe { r_crypto_set_key(crypto, key.as_ptr(), key.len(), mode, is_decryption) }
}
pub fn set_iv(crypto: *const c_void, iv: &[u8]) -> bool {
    unsafe { r_crypto_set_iv(crypto, iv.as_ptr(), iv.len()) }
}

pub fn update(crypto: *const c_void, buf: &[u8]) {
    unsafe { r_crypto_update(crypto, buf.as_ptr(), buf.len()) }
}
pub fn finish(crypto: *const c_void, buf: &[u8]) {
    unsafe { r_crypto_final(crypto, buf.as_ptr(), buf.len()) }
}
