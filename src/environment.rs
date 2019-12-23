/*
 * environment.rs: Linking all rair parts together into 1 module.
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
use std::collections::HashMap;
use std::mem;

pub type StrFn<T> = fn(&str, &str, &Environment<T>, &mut T) -> bool;
pub type U64Fn<T> = fn(&str, u64, &Environment<T>, &mut T) -> bool;
pub type I64Fn<T> = fn(&str, i64, &Environment<T>, &mut T) -> bool;
pub type BoolFn<T> = fn(&str, bool, &Environment<T>, &mut T) -> bool;

#[derive(Debug, PartialEq)]
pub enum EnvErr {
    NotFound,
    DifferentType,
    CbFailed,
    AlreadyExist,
}
struct EnvStr<T> {
    data: String,
    default: String,
    cb: Option<StrFn<T>>,
}

struct EnvU64<T> {
    data: u64,
    default: u64,
    cb: Option<U64Fn<T>>,
}
struct EnvI64<T> {
    data: i64,
    default: i64,
    cb: Option<I64Fn<T>>,
}
struct EnvBool<T> {
    data: bool,
    default: bool,
    cb: Option<BoolFn<T>>,
}
enum EnvMetaData<T> {
    Str(EnvStr<T>),
    U64(EnvU64<T>),
    I64(EnvI64<T>),
    Bool(EnvBool<T>),
}

impl<T> EnvMetaData<T> {
    fn as_str(&self) -> Option<&EnvStr<T>> {
        if let EnvMetaData::Str(s) = self {
            return Some(s);
        }
        return None;
    }
    fn as_u64(&self) -> Option<&EnvU64<T>> {
        if let EnvMetaData::U64(u) = self {
            return Some(u);
        }
        return None;
    }
    fn as_i64(&self) -> Option<&EnvI64<T>> {
        if let EnvMetaData::I64(i) = self {
            return Some(i);
        }
        return None;
    }
    fn as_bool(&self) -> Option<&EnvBool<T>> {
        if let EnvMetaData::Bool(b) = self {
            return Some(b);
        }
        return None;
    }

    fn mut_str(&mut self) -> Option<&mut EnvStr<T>> {
        if let EnvMetaData::Str(s) = self {
            return Some(s);
        }
        return None;
    }
    fn mut_u64(&mut self) -> Option<&mut EnvU64<T>> {
        if let EnvMetaData::U64(u) = self {
            return Some(u);
        }
        return None;
    }
    fn mut_i64(&mut self) -> Option<&mut EnvI64<T>> {
        if let EnvMetaData::I64(i) = self {
            return Some(i);
        }
        return None;
    }
    fn mut_bool(&mut self) -> Option<&mut EnvBool<T>> {
        if let EnvMetaData::Bool(b) = self {
            return Some(b);
        }
        return None;
    }
}
#[derive(PartialEq, Debug)]
pub enum EnvData<'a> {
    Str(&'a str),
    U64(u64),
    I64(i64),
    Bool(bool),
}

#[derive(Default)]
pub struct Environment<T> {
    data: HashMap<String, EnvMetaData<T>>,
}

impl<T> Environment<T> {
    pub fn new() -> Self {
        Environment { data: HashMap::new() }
    }
    // All exec_*_cb function are guaranteed to be running on the correct type
    fn exec_str_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data.get(key).unwrap().as_str().unwrap();
        let val = &meta.data;
        if let Some(cb) = meta.cb {
            return cb(key, val, self, data);
        }
        return true;
    }

    pub fn add_str_with_cb(&mut self, key: &str, val: &str, data: &mut T, cb: StrFn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvStr {
            data: val.to_string(),
            default: val.to_string(),
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::Str(meta));
        if !self.exec_str_cb(key, data) {
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

    pub fn set_str(&mut self, key: &str, value: &str, data: &mut T) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value.to_string();
        if let Some(s) = meta.mut_str() {
            mem::swap(&mut tmp, &mut s.data);
            if !self.exec_str_cb(key, data) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_str().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::DifferentType);
        }
    }

    pub fn is_str(&self, key: &str) -> bool {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return false,
        };
        return meta.as_str().is_some();
    }

    fn exec_u64_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data.get(key).unwrap().as_u64().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, data);
        }
        return true;
    }

    pub fn add_u64_with_cb(&mut self, key: &str, val: u64, data: &mut T, cb: U64Fn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvU64 {
            data: val,
            default: val,
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::U64(meta));
        if !self.exec_u64_cb(key, data) {
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

    pub fn set_u64(&mut self, key: &str, value: u64, data: &mut T) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value;
        if let Some(s) = meta.mut_u64() {
            mem::swap(&mut tmp, &mut s.data);
            if !self.exec_u64_cb(key, data) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_u64().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::DifferentType);
        }
    }

    pub fn is_u64(&self, key: &str) -> bool {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return false,
        };
        return meta.as_u64().is_some();
    }

    fn exec_i64_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data.get(key).unwrap().as_i64().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, data);
        }
        return true;
    }

    pub fn add_i64_with_cb(&mut self, key: &str, val: i64, data: &mut T, cb: I64Fn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvI64 {
            data: val,
            default: val,
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::I64(meta));
        if !self.exec_i64_cb(key, data) {
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

    pub fn set_i64(&mut self, key: &str, value: i64, data: &mut T) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value;
        if let Some(s) = meta.mut_i64() {
            mem::swap(&mut tmp, &mut s.data);
            if !self.exec_i64_cb(key, data) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_i64().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::DifferentType);
        }
    }

    pub fn is_i64(&self, key: &str) -> bool {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return false,
        };
        return meta.as_i64().is_some();
    }

    fn exec_bool_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data.get(key).unwrap().as_bool().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, data);
        }
        return true;
    }

    pub fn add_bool_with_cb(&mut self, key: &str, val: bool, data: &mut T, cb: BoolFn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvBool {
            data: val,
            default: val,
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::Bool(meta));
        if !self.exec_bool_cb(key, data) {
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

    pub fn set_bool(&mut self, key: &str, value: bool, data: &mut T) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value;
        if let Some(s) = meta.mut_bool() {
            mem::swap(&mut tmp, &mut s.data);
            if !self.exec_bool_cb(key, data) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_bool().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::DifferentType);
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

#[cfg(test)]
mod test_environment {
    use super::*;
    fn even_str(_: &str, value: &str, _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        return value.len() % 2 == 0;
    }
    fn even_u64(_: &str, value: u64, _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        return value % 2 == 0;
    }
    fn negative_i64(_: &str, value: i64, _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        return value < 0;
    }
    fn always_false(_: &str, value: bool, _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        return !value;
    }
    fn prep_env() -> Environment<Option<()>> {
        let mut data = None;
        let mut env = Environment::new();
        env.add_str("s1", "value1").unwrap();
        env.add_str_with_cb("s2", "value2", &mut data, even_str).unwrap();
        env.add_u64("u1", 1).unwrap();
        env.add_u64_with_cb("u2", 2, &mut data, even_u64).unwrap();
        env.add_i64("i1", 1).unwrap();
        env.add_i64_with_cb("i2", -1, &mut data, negative_i64).unwrap();
        env.add_bool("b1", true).unwrap();
        env.add_bool_with_cb("b2", false, &mut data, always_false).unwrap();

        return env;
    }
    #[test]
    fn test_str() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_str_with_cb("s03", "value02", &mut data, even_str).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_str("s1", "v3").err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_str_with_cb("s1", "value1", &mut data, even_str).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_str("s1"), true);
        assert_eq!(env.is_str("u1"), false);
        assert_eq!(env.get_str("s1").unwrap(), "value1");
        assert_eq!(env.get_str("s2").unwrap(), "value2");
        assert_eq!(env.get_str("s3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_str("u1").err().unwrap(), EnvErr::DifferentType);
        env.set_str("s1", "newvalue1", &mut data).unwrap();
        assert_eq!(env.get_str("s1").unwrap(), "newvalue1");
        env.set_str("s2", "newvalue02", &mut data).unwrap();
        assert_eq!(env.get_str("s2").unwrap(), "newvalue02");
        assert_eq!(env.set_str("s2", "tmp", &mut data).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.get_str("s2").unwrap(), "newvalue02");
        assert_eq!(env.set_str("s3", "tmp", &mut data).err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.set_str("u1", "tmp", &mut data).err().unwrap(), EnvErr::DifferentType);
        assert_eq!(env.get("s1").unwrap(), EnvData::Str("newvalue1"));
        assert_eq!(env.get("s3"), None);
    }
    #[test]
    fn test_u64() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_u64_with_cb("u3", 3, &mut data, even_u64).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_u64("u2", 5).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_u64_with_cb("s1", 4, &mut data, even_u64).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_u64("u1"), true);
        assert_eq!(env.is_u64("s1"), false);
        assert_eq!(env.get_u64("u1").unwrap(), 1);
        assert_eq!(env.get_u64("u2").unwrap(), 2);
        assert_eq!(env.get_u64("u3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_u64("s1").err().unwrap(), EnvErr::DifferentType);
        env.set_u64("u1", 8, &mut data).unwrap();
        assert_eq!(env.get_u64("u1").unwrap(), 8);
        env.set_u64("u2", 4, &mut data).unwrap();
        assert_eq!(env.get_u64("u2").unwrap(), 4);
        assert_eq!(env.set_u64("u2", 7, &mut data).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.get_u64("u2").unwrap(), 4);
        assert_eq!(env.set_u64("u3", 5, &mut data).err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.set_u64("s1", 3, &mut data).err().unwrap(), EnvErr::DifferentType);
        assert_eq!(env.get("u1").unwrap(), EnvData::U64(8));
    }
    #[test]
    fn test_i64() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_i64_with_cb("i3", 3, &mut data, negative_i64).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_i64("i2", 5).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_i64_with_cb("s1", 4, &mut data, negative_i64).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_i64("i1"), true);
        assert_eq!(env.is_i64("u1"), false);
        assert_eq!(env.get_i64("i1").unwrap(), 1);
        assert_eq!(env.get_i64("i2").unwrap(), -1);
        assert_eq!(env.get_i64("u3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_i64("s1").err().unwrap(), EnvErr::DifferentType);
        env.set_i64("i1", 8, &mut data).unwrap();
        assert_eq!(env.get_i64("i1").unwrap(), 8);
        env.set_i64("i2", -4, &mut data).unwrap();
        assert_eq!(env.get_i64("i2").unwrap(), -4);
        assert_eq!(env.set_i64("i2", 7, &mut data).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.get_i64("i2").unwrap(), -4);
        assert_eq!(env.set_i64("i3", 5, &mut data).err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.set_i64("s1", 3, &mut data).err().unwrap(), EnvErr::DifferentType);
        assert_eq!(env.get("i1").unwrap(), EnvData::I64(8));
    }
    #[test]
    fn test_bool() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_bool_with_cb("b3", true, &mut data, always_false).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_bool("b2", true).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_bool_with_cb("b1", false, &mut data, always_false).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_bool("b1"), true);
        assert_eq!(env.is_bool("u1"), false);
        assert_eq!(env.get_bool("b1").unwrap(), true);
        assert_eq!(env.get_bool("b2").unwrap(), false);
        assert_eq!(env.get_bool("u3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_bool("s1").err().unwrap(), EnvErr::DifferentType);
        env.set_bool("b1", false, &mut data).unwrap();
        assert_eq!(env.get_bool("b1").unwrap(), false);
        assert_eq!(env.set_bool("b2", true, &mut data).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.get_bool("b2").unwrap(), false);
        assert_eq!(env.set_bool("b3", true, &mut data).err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.set_bool("s1", false, &mut data).err().unwrap(), EnvErr::DifferentType);
        assert_eq!(env.get("b1").unwrap(), EnvData::Bool(false));
    }

}
