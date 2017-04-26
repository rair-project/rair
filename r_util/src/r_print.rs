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
//TODO delete the whole file ^_^
extern crate libc;
use libc::*;
use std::io::{self, Write};
use std::ffi::CStr;
use std::process;
#[derive(PartialEq,Clone)]
pub enum OutputFormat {
    Json,
    Command,
    Ssh, // used by rahash -k option
    None,
}
#[link(name = "r_util")]
extern "C" {
    fn r_print_progressbar(rprint: *const c_void, pc: i32, cols: i32);
    fn r_print_randomart(digest: *const u8, digest_len: usize, addr: usize) -> *const c_char;
    fn r_print_new() -> *const c_void;
    fn r_print_hexdump(pr: *const c_void, addr: usize, buf: *const u8, len: usize, base: usize, step: bool);
}
pub fn progressbar(rprint: *const c_void, pc: i32, cols: i32) {
    unsafe { r_print_progressbar(rprint, pc, cols) }
}
pub fn randomart(digest: &[u8], addr: usize) -> String {
    let x = unsafe { r_print_randomart(digest.as_ptr(), digest.len(), addr) };
    unsafe { CStr::from_ptr(x).to_string_lossy().into_owned() }
}
pub fn report(error: &str) -> ! {
    writeln!(&mut io::stderr(), "{}", error).unwrap();
    process::exit(1);
}
pub fn new<'a>() -> &'a c_void{
    unsafe{&*r_print_new()}
}
pub fn hexdump(pr: &c_void, addr:usize, buf:&[u8], base: usize, step: bool) {
    unsafe{r_print_hexdump(pr, addr, buf.as_ptr(), buf.len(), base, step)}
}
