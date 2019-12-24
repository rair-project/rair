/*
 * environment.rs: Linking all rair configuration parts together into place.
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
use super::err::*;
use super::metadata::*;
use std::collections::HashMap;
use std::mem;

#[derive(PartialEq, Debug)]
pub enum EnvData<'a> {
    Str(&'a str),
    U64(u64),
    I64(i64),
    Bool(bool),
    Color(u8, u8, u8),
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

    pub fn add_str_with_cb(&mut self, key: &str, val: &str, help: &str, data: &mut T, cb: StrFn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvStr {
            data: val.to_string(),
            default: val.to_string(),
            help: help.to_string(),
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::Str(meta));
        if !self.exec_str_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_str(&mut self, key: &str, val: &str, help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvStr {
            data: val.to_string(),
            default: val.to_string(),
            help: help.to_string(),
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

    pub fn add_u64_with_cb(&mut self, key: &str, val: u64, help: &str, data: &mut T, cb: U64Fn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvU64 {
            data: val,
            default: val,
            help: help.to_string(),
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::U64(meta));
        if !self.exec_u64_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_u64(&mut self, key: &str, val: u64, help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvU64 {
            data: val,
            default: val,
            help: help.to_string(),
            cb: None,
        };
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

    pub fn add_i64_with_cb(&mut self, key: &str, val: i64, help: &str, data: &mut T, cb: I64Fn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvI64 {
            data: val,
            default: val,
            help: help.to_string(),
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::I64(meta));
        if !self.exec_i64_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_i64(&mut self, key: &str, val: i64, help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvI64 {
            data: val,
            default: val,
            help: help.to_string(),
            cb: None,
        };
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

    pub fn add_bool_with_cb(&mut self, key: &str, val: bool, help: &str, data: &mut T, cb: BoolFn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvBool {
            data: val,
            default: val,
            help: help.to_string(),
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::Bool(meta));
        if !self.exec_bool_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_bool(&mut self, key: &str, val: bool, help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvBool {
            data: val,
            default: val,
            help: help.to_string(),
            cb: None,
        };
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

    fn exec_color_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data.get(key).unwrap().as_color().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, data);
        }
        return true;
    }

    pub fn add_color_with_cb(&mut self, key: &str, val: (u8, u8, u8), help: &str, data: &mut T, cb: ColorFn<T>) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvColor {
            data: val,
            default: val,
            help: help.to_string(),
            cb: Some(cb),
        };
        self.data.insert(key.to_string(), EnvMetaData::Color(meta));
        if !self.exec_color_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        return Ok(());
    }

    pub fn add_color(&mut self, key: &str, val: (u8, u8, u8), help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvColor {
            data: val,
            default: val,
            help: help.to_string(),
            cb: None,
        };
        self.data.insert(key.to_string(), EnvMetaData::Color(meta));
        return Ok(());
    }

    pub fn get_color(&self, key: &str) -> Result<(u8, u8, u8), EnvErr> {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        match meta.as_color() {
            Some(s) => return Ok(s.data),
            None => return Err(EnvErr::DifferentType),
        };
    }

    pub fn set_color(&mut self, key: &str, value: (u8, u8, u8), data: &mut T) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        let mut tmp = value;
        if let Some(s) = meta.mut_color() {
            mem::swap(&mut tmp, &mut s.data);
            if !self.exec_color_cb(key, data) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_color().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            return Ok(());
        } else {
            return Err(EnvErr::DifferentType);
        }
    }

    pub fn is_color(&self, key: &str) -> bool {
        let meta = match self.data.get(key) {
            Some(meta) => meta,
            None => return false,
        };
        return meta.as_color().is_some();
    }

    pub fn reset(&mut self, key: &str, data: &mut T) -> Result<(), EnvErr> {
        let meta = match self.data.get_mut(key) {
            Some(meta) => meta,
            None => return Err(EnvErr::NotFound),
        };
        match meta {
            EnvMetaData::Bool(b) => {
                b.data = b.default;
                self.exec_bool_cb(key, data);
            }
            EnvMetaData::Color(c) => {
                c.data = c.default;
                self.exec_color_cb(key, data);
            }
            EnvMetaData::I64(i) => {
                i.data = i.default;
                self.exec_i64_cb(key, data);
            }
            EnvMetaData::U64(u) => {
                u.data = u.default;
                self.exec_u64_cb(key, data);
            }
            EnvMetaData::Str(s) => {
                s.data = s.default.clone();
                self.exec_str_cb(key, data);
            }
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
            EnvMetaData::Color(c) => Some(EnvData::Color(c.data.0, c.data.1, c.data.2)),
        };
    }
    pub fn get_help(&self, key: &str) -> Option<&str> {
        let meta = self.data.get(key)?;
        return match meta {
            EnvMetaData::Bool(b) => Some(&b.help),
            EnvMetaData::I64(i) => Some(&i.help),
            EnvMetaData::U64(u) => Some(&u.help),
            EnvMetaData::Str(s) => Some(&s.help),
            EnvMetaData::Color(c) => Some(&c.help),
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
    fn grayscale(_: &str, color: (u8, u8, u8), _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        let (r, g, b) = color;
        return r == g && g == b;
    }
    fn prep_env() -> Environment<Option<()>> {
        let mut data = None;
        let mut env = Environment::new();
        env.add_str("s1", "value1", "First String").unwrap();
        env.add_str_with_cb("s2", "value2", "Second String", &mut data, even_str).unwrap();
        env.add_u64("u1", 1, "First U64").unwrap();
        env.add_u64_with_cb("u2", 2, "Second U64", &mut data, even_u64).unwrap();
        env.add_i64("i1", 1, "First I64").unwrap();
        env.add_i64_with_cb("i2", -1, "Second I64", &mut data, negative_i64).unwrap();
        env.add_bool("b1", true, "First Bool").unwrap();
        env.add_bool_with_cb("b2", false, "Second Bool", &mut data, always_false).unwrap();
        env.add_color("c1", (31, 33, 37), "First Color").unwrap();
        env.add_color_with_cb("c2", (50, 50, 50), "Second Color", &mut data, grayscale).unwrap();
        return env;
    }
    #[test]
    fn test_str() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_str_with_cb("s03", "value02", "", &mut data, even_str).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_str("s1", "v3", "").err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_str_with_cb("s1", "value1", "", &mut data, even_str).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_str("s1"), true);
        assert_eq!(env.is_str("u1"), false);
        assert_eq!(env.is_str("s3"), false);
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
        env.reset("s1", &mut data).unwrap();
        assert_eq!(env.get_str("s1").unwrap(), "value1");
        assert_eq!(env.reset("s3", &mut data).err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_help("s3"), None);
        assert_eq!(env.get_help("s1").unwrap(), "First String");
    }
    #[test]
    fn test_u64() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_u64_with_cb("u3", 3, "", &mut data, even_u64).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_u64("u2", 5, "").err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_u64_with_cb("s1", 4, "", &mut data, even_u64).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_u64("u1"), true);
        assert_eq!(env.is_u64("s1"), false);
        assert_eq!(env.is_u64("u3"), false);
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
        env.reset("u1", &mut data).unwrap();
        assert_eq!(env.get_u64("u1").unwrap(), 1);
        assert_eq!(env.get_help("u1").unwrap(), "First U64");
    }
    #[test]
    fn test_i64() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_i64_with_cb("i3", 3, "", &mut data, negative_i64).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_i64("i2", 5, "").err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_i64_with_cb("s1", 4, "", &mut data, negative_i64).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_i64("i1"), true);
        assert_eq!(env.is_i64("u1"), false);
        assert_eq!(env.is_i64("i3"), false);
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
        env.reset("i1", &mut data).unwrap();
        assert_eq!(env.get_i64("i1").unwrap(), 1);
        assert_eq!(env.get_help("i1").unwrap(), "First I64");
    }
    #[test]
    fn test_bool() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_bool_with_cb("b3", true, "", &mut data, always_false).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_bool("b2", true, "").err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_bool_with_cb("b1", false, "", &mut data, always_false).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_bool("b1"), true);
        assert_eq!(env.is_bool("u1"), false);
        assert_eq!(env.is_bool("b3"), false);
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
        env.reset("b1", &mut data).unwrap();
        assert_eq!(env.get_bool("b1").unwrap(), true);
        assert_eq!(env.get_help("b1").unwrap(), "First Bool");
    }

    #[test]
    fn test_color() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(env.add_color_with_cb("c3", (50, 60, 70), "", &mut data, grayscale).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.add_color("c2", (20, 30, 40), "").err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.add_color_with_cb("c1", (100, 100, 100), "", &mut data, grayscale).err().unwrap(), EnvErr::AlreadyExist);
        assert_eq!(env.is_color("c1"), true);
        assert_eq!(env.is_color("u1"), false);
        assert_eq!(env.is_color("c3"), false);
        /**/
        assert_eq!(env.get_color("c1").unwrap(), (31, 33, 37));
        assert_eq!(env.get_color("c2").unwrap(), (50, 50, 50));
        assert_eq!(env.get_color("c3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_color("s1").err().unwrap(), EnvErr::DifferentType);
        env.set_color("c1", (20, 40, 60), &mut data).unwrap();
        assert_eq!(env.get_color("c1").unwrap(), (20, 40, 60));
        assert_eq!(env.set_color("c2", (5, 10, 15), &mut data).err().unwrap(), EnvErr::CbFailed);
        assert_eq!(env.get_color("c2").unwrap(), (50, 50, 50));
        assert_eq!(env.set_color("c3", (50, 50, 50), &mut data).err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.set_color("s1", (50, 50, 50), &mut data).err().unwrap(), EnvErr::DifferentType);
        assert_eq!(env.get("c1").unwrap(), EnvData::Color(20, 40, 60));
        env.reset("c1", &mut data).unwrap();
        assert_eq!(env.get_color("c1").unwrap(), (31, 33, 37));
        assert_eq!(env.get_help("c1").unwrap(), "First Color");
    }
}
