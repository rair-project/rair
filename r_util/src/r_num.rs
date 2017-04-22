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
use std::ffi::{CString,CStr};
use std::ptr;
#[link(name = "r_util")]
extern "C" {
    fn r_num_new(cb: *const c_void, cb2: *const c_void, ptr: *const c_void) -> *mut RNum;
    fn r_num_calc(rnum: *const RNum, STR: *const c_char, err: *const *const c_char) -> u64;
}
struct RNumCalcValue {
    d:f64,
    n:u64,
}
struct RNumCalc {
    curr_tok: u32, //TODO turn this later into RNumCalcToken
    number_value: RNumCalcValue,
    string_value: [char;1024], //TODO turn this into string
    errors: i32, //TODO make use of rust error handling
    oc: char,
    calr_err: *const c_char,
    calc_i: i32, //TODO check this later
    calc_buf: *const c_char,
    calc_len: usize,
}
pub struct RNum {
    callback: *const c_void, //ut64 (*callback)(struct r_num_t *userptr, const char *str, int *ok);
    cb_from_value: *const c_void, //const char *(*cb_from_value)(struct r_num_t *userptr, ut64 value, int *ok);
    value: u64,
    fvalue: f64,
    userptr: *const c_void,
    dbz: bool, //division by zero
    nc: RNumCalc,
}
impl RNum {
    pub fn new<'a>(cb: Option<*const c_void>, cb2: Option<*const c_void>, ptr: Option<*const c_void>) -> &'a mut RNum {
        let c_cb = match cb {
            Some(x) => x,
            None => ptr::null(),
        };
        let c_cb2 = match cb2 {
            Some(x) => x,
            None => ptr::null(),
        };
        let c_ptr = match ptr {
            Some(x) => x,
            None => ptr::null(),
        };
        unsafe{&mut *r_num_new(c_cb, c_cb2, c_ptr)}
    }
    //TODO Maybe we should turn all results into some sort of Result<whatever, &str>
    pub fn math(&mut self, string: &str) -> Result<u64, String> {
        if string.is_empty() {
            return Ok(0);
        }
        self.dbz = false;
        self.value = self.calc(string)?;
        return Ok(self.value);
    }
    pub fn calc(&self, string: &str) -> Result<u64, String> {
        let err: *const c_char = ptr::null();
        let err_string;
        let cstring = CString::new(string).unwrap();
        let ret = unsafe{r_num_calc(self, cstring.as_ptr(), &err)};
        if err == ptr::null() {
            return Ok(ret);
        } else {
            err_string = unsafe{CStr::from_ptr(err).to_string_lossy().into_owned()};
            return Err(err_string);
        }
    }
}
