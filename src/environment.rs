/*
 * core.rs: Linking all rair parts together into 1 module.
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
use core::Core;
use std::collections::HashMap;
use std::mem;

type StrFn = fn(&str, &str, &Environment, &mut Core) -> bool;
type U64Fn = fn(&str, u64, &Environment, &mut Core) -> bool;
type I64Fn = fn(&str, i64, &Environment, &mut Core) -> bool;
type BoolFn = fn(&str, bool, &Environment, &mut Core) -> bool;

pub enum EnvErr {
    NotFound,
    DifferentType,
    CbFailed,
    AlreadyExist,
}
struct EnvStr {
    data: String,
    default: String,
    cb: Option<StrFn>,
}

struct EnvU64 {
    data: u64,
    default: u64,
    cb: Option<U64Fn>,
}
struct EnvI64 {
    data: i64,
    default: i64,
    cb: Option<I64Fn>,
}
struct EnvBool {
    data: bool,
    default: bool,
    cb: Option<BoolFn>,
}
enum EnvMetaData {
    Str(EnvStr),
    U64(EnvU64),
    I64(EnvI64),
    Bool(EnvBool),
}

impl EnvMetaData {
    fn as_str(&self) -> Option<&EnvStr> {
        if let EnvMetaData::Str(s) = self {
            return Some(s);
        }
        return None;
    }
    fn as_u64(&self) -> Option<&EnvU64> {
        if let EnvMetaData::U64(u) = self {
            return Some(u);
        }
        return None;
    }
    fn as_i64(&self) -> Option<&EnvI64> {
        if let EnvMetaData::I64(i) = self {
            return Some(i);
        }
        return None;
    }
    fn as_bool(&self) -> Option<&EnvBool> {
        if let EnvMetaData::Bool(b) = self {
            return Some(b);
        }
        return None;
    }

    fn mut_str(&mut self) -> Option<&mut EnvStr> {
        if let EnvMetaData::Str(s) = self {
            return Some(s);
        }
        return None;
    }
    fn mut_u64(&mut self) -> Option<&mut EnvU64> {
        if let EnvMetaData::U64(u) = self {
            return Some(u);
        }
        return None;
    }
    fn mut_i64(&mut self) -> Option<&mut EnvI64> {
        if let EnvMetaData::I64(i) = self {
            return Some(i);
        }
        return None;
    }
    fn mut_bool(&mut self) -> Option<&mut EnvBool> {
        if let EnvMetaData::Bool(b) = self {
            return Some(b);
        }
        return None;
    }
}
pub enum EnvData<'a> {
    Str(&'a str),
    U64(u64),
    I64(i64),
    Bool(bool),
}
#[derive(Default)]
pub struct Environment {
    data: HashMap<String, EnvMetaData>,
}

impl Environment {
    pub fn new() -> Self {
        Default::default()
    }
    // All exec_*_cb function are guaranteed to be running on the correct type
    fn exec_str_cb(&self, key: &str, core: &mut Core) -> bool {
        let meta = self.data.get(key).unwrap().as_str().unwrap();
        let val = &meta.data;
        if let Some(cb) = meta.cb {
            return cb(key, val, self, core);
        }
        return true;
    }

    pub fn add_str_with_cb(&mut self, key: &str, val: &str, core: &mut Core, cb: StrFn) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvStr {
            data: val.to_string(),
            default: val.to_string(),
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::Str(meta));
        if !self.exec_str_cb(key, core) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_str(&mut self, key: &str, val: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvStr {
            data: val.to_string(),
            default: val.to_string(),
            cb: None,
        };
        self.data.insert(key.to_string(), EnvMetaData::Str(meta));
        return Ok(());
    }

    pub fn get_str(&self, key: &str) -> Result<&str, EnvErr> {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        match meta.as_str() {
            Some(s) => return Ok(&s.data),
            None => return Err(EnvErr::DifferentType),
        };
    }

    pub fn set_str(&mut self, key: &str, value: &str, core: &mut Core) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value.to_string();
        if let Some(s) = meta.mut_str() {
            mem::swap(&mut tmp, &mut s.data);
            drop(s);
            drop(meta);
            if !self.exec_str_cb(key, core) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_str().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::NotFound);
        }
    }

    pub fn is_str(&self, key: &str) -> bool {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return false,
        };
        return meta.as_str().is_some();
    }

    fn exec_u64_cb(&self, key: &str, core: &mut Core) -> bool {
        let meta = self.data.get(key).unwrap().as_u64().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, core);
        }
        return true;
    }

    pub fn add_u64_with_cb(&mut self, key: &str, val: u64, core: &mut Core, cb: U64Fn) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvU64 {
            data: val,
            default: val,
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::U64(meta));
        if !self.exec_u64_cb(key, core) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_u64(&mut self, key: &str, val: u64) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvU64 { data: val, default: val, cb: None };
        self.data.insert(key.to_string(), EnvMetaData::U64(meta));
        return Ok(());
    }

    pub fn get_u64(&self, key: &str) -> Result<u64, EnvErr> {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        match meta.as_u64() {
            Some(s) => return Ok(s.data),
            None => return Err(EnvErr::DifferentType),
        };
    }

    pub fn set_u64(&mut self, key: &str, value: u64, core: &mut Core) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value;
        if let Some(s) = meta.mut_u64() {
            mem::swap(&mut tmp, &mut s.data);
            drop(s);
            drop(meta);
            if !self.exec_u64_cb(key, core) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_u64().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::NotFound);
        }
    }

    pub fn is_u64(&self, key: &str) -> bool {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return false,
        };
        return meta.as_u64().is_some();
    }

    fn exec_i64_cb(&self, key: &str, core: &mut Core) -> bool {
        let meta = self.data.get(key).unwrap().as_i64().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, core);
        }
        return true;
    }

    pub fn add_i64_with_cb(&mut self, key: &str, val: i64, core: &mut Core, cb: I64Fn) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvI64 {
            data: val,
            default: val,
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::I64(meta));
        if !self.exec_i64_cb(key, core) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_i64(&mut self, key: &str, val: i64) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvI64 { data: val, default: val, cb: None };
        self.data.insert(key.to_string(), EnvMetaData::I64(meta));
        return Ok(());
    }

    pub fn get_i64(&self, key: &str) -> Result<i64, EnvErr> {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        match meta.as_i64() {
            Some(s) => return Ok(s.data),
            None => return Err(EnvErr::DifferentType),
        };
    }

    pub fn set_i64(&mut self, key: &str, value: i64, core: &mut Core) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value;
        if let Some(s) = meta.mut_i64() {
            mem::swap(&mut tmp, &mut s.data);
            drop(s);
            drop(meta);
            if !self.exec_i64_cb(key, core) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_i64().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::NotFound);
        }
    }

    pub fn is_i64(&self, key: &str) -> bool {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return false,
        };
        return meta.as_i64().is_some();
    }

    fn exec_bool_cb(&self, key: &str, core: &mut Core) -> bool {
        let meta = self.data.get(key).unwrap().as_bool().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, core);
        }
        return true;
    }

    pub fn add_bool_with_cb(&mut self, key: &str, val: bool, core: &mut Core, cb: BoolFn) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvBool {
            data: val,
            default: val,
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::Bool(meta));
        if !self.exec_bool_cb(key, core) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_bool(&mut self, key: &str, val: bool) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvBool { data: val, default: val, cb: None };
        self.data.insert(key.to_string(), EnvMetaData::Bool(meta));
        return Ok(());
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, EnvErr> {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        match meta.as_bool() {
            Some(s) => return Ok(s.data),
            None => return Err(EnvErr::DifferentType),
        };
    }

    pub fn set_bool(&mut self, key: &str, value: bool, core: &mut Core) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value;
        if let Some(s) = meta.mut_bool() {
            mem::swap(&mut tmp, &mut s.data);
            drop(s);
            drop(meta);
            if !self.exec_bool_cb(key, core) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_bool().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::NotFound);
        }
    }

    pub fn is_bool(&self, key: &str) -> bool {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return false,
        };
        return meta.as_bool().is_some();
    }

    pub fn reset(&mut self, key: &str) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        match meta {
            EnvMetaData::Bool(b) => b.data = b.default,
            EnvMetaData::I64(i) => i.data = i.default,
            EnvMetaData::U64(u) => u.data = u.default,
            EnvMetaData::Str(s) => s.data = s.default.clone(),
        }
        return Ok(());
    }
    pub fn get(&self, key: &str) -> Option<EnvData> {
        let meta = self.data.get(key)?;
        return match meta {
            EnvMetaData::Bool(b) => Some(EnvData::Bool(b.data)),
            EnvMetaData::I64(i) => Some(EnvData::I64(i.data)),
            EnvMetaData::U64(u) => Some(EnvData::U64(u.data)),
            EnvMetaData::Str(s) => Some(EnvData::Str(&s.data)),
        };
    }
}
