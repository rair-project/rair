//! Linking all rair configuration parts together into place.

use super::err::EnvErr;
use super::metadata::{
    BoolFn, ColorFn, EnvBool, EnvColor, EnvI64, EnvMetaData, EnvStr, EnvU64, I64Fn, StrFn, U64Fn,
};
use core::mem;
use std::collections::HashMap;
#[derive(PartialEq, Debug)]
pub enum EnvData<'a> {
    Str(&'a str),
    U64(u64),
    I64(i64),
    Bool(bool),
    Color(u8, u8, u8),
}
impl<'a, T> From<&'a EnvMetaData<T>> for EnvData<'a> {
    fn from(meta: &'a EnvMetaData<T>) -> Self {
        match meta {
            EnvMetaData::Str(s) => EnvData::Str(&s.data),
            EnvMetaData::I64(i) => EnvData::I64(i.data),
            EnvMetaData::U64(u) => EnvData::U64(u.data),
            EnvMetaData::Bool(u) => EnvData::Bool(u.data),
            EnvMetaData::Color(c) => EnvData::Color(c.data.0, c.data.1, c.data.2),
        }
    }
}
#[derive(Default)]
pub struct Environment<T> {
    data: HashMap<String, EnvMetaData<T>>,
}

impl<T> Environment<T> {
    #[must_use]
    pub fn new() -> Self {
        Environment {
            data: HashMap::new(),
        }
    }
    // All exec_*_cb function are guaranteed to be running on the correct type
    fn exec_str_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data[key].as_str().unwrap();
        let val = &meta.data;
        if let Some(cb) = meta.cb {
            return cb(key, val, self, data);
        }
        true
    }

    pub fn add_str_with_cb(
        &mut self,
        key: &str,
        val: &str,
        help: &str,
        data: &mut T,
        cb: StrFn<T>,
    ) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvStr {
            data: val.to_owned(),
            default: val.to_owned(),
            help: help.to_owned(),
            cb: Some(cb),
        };
        self.data.insert(key.to_owned(), EnvMetaData::Str(meta));
        if !self.exec_str_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        Ok(())
    }

    pub fn add_str(&mut self, key: &str, val: &str, help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvStr {
            data: val.to_owned(),
            default: val.to_owned(),
            help: help.to_owned(),
            cb: None,
        };
        self.data.insert(key.to_owned(), EnvMetaData::Str(meta));
        Ok(())
    }

    pub fn get_str(&self, key: &str) -> Result<&str, EnvErr> {
        let Some(meta) = self.data.get(key) else {
            return Err(EnvErr::NotFound);
        };
        match meta.as_str() {
            Some(s) => Ok(&s.data),
            None => Err(EnvErr::DifferentType),
        }
    }

    pub fn set_str(&mut self, key: &str, value: &str, data: &mut T) -> Result<(), EnvErr> {
        let Some(meta) = self.data.get_mut(key) else {
            return Err(EnvErr::NotFound);
        };
        let mut tmp = value.to_owned();
        if let Some(s) = meta.mut_str() {
            mem::swap(&mut tmp, &mut s.data);
            if !self.exec_str_cb(key, data) {
                //restore old data
                let meta = self.data.get_mut(key).unwrap();
                let s = meta.mut_str().unwrap();
                mem::swap(&mut s.data, &mut tmp);
                return Err(EnvErr::CbFailed);
            }
            Ok(())
        } else {
            Err(EnvErr::DifferentType)
        }
    }

    #[must_use]
    pub fn is_str(&self, key: &str) -> bool {
        let Some(meta) = self.data.get(key) else {
            return false;
        };
        meta.as_str().is_some()
    }

    fn exec_u64_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data[key].as_u64().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, data);
        }
        true
    }

    pub fn add_u64_with_cb(
        &mut self,
        key: &str,
        val: u64,
        help: &str,
        data: &mut T,
        cb: U64Fn<T>,
    ) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvU64 {
            data: val,
            default: val,
            help: help.to_owned(),
            cb: Some(cb),
        };
        self.data.insert(key.to_owned(), EnvMetaData::U64(meta));
        if !self.exec_u64_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        Ok(())
    }

    pub fn add_u64(&mut self, key: &str, val: u64, help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvU64 {
            data: val,
            default: val,
            help: help.to_owned(),
            cb: None,
        };
        self.data.insert(key.to_owned(), EnvMetaData::U64(meta));
        Ok(())
    }

    pub fn get_u64(&self, key: &str) -> Result<u64, EnvErr> {
        let Some(meta) = self.data.get(key) else {
            return Err(EnvErr::NotFound);
        };
        match meta.as_u64() {
            Some(s) => Ok(s.data),
            None => Err(EnvErr::DifferentType),
        }
    }

    pub fn set_u64(&mut self, key: &str, value: u64, data: &mut T) -> Result<(), EnvErr> {
        let Some(meta) = self.data.get_mut(key) else {
            return Err(EnvErr::NotFound);
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
            Ok(())
        } else {
            Err(EnvErr::DifferentType)
        }
    }

    #[must_use]
    pub fn is_u64(&self, key: &str) -> bool {
        let Some(meta) = self.data.get(key) else {
            return false;
        };
        meta.as_u64().is_some()
    }

    fn exec_i64_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data[key].as_i64().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, data);
        }
        true
    }

    pub fn add_i64_with_cb(
        &mut self,
        key: &str,
        val: i64,
        help: &str,
        data: &mut T,
        cb: I64Fn<T>,
    ) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvI64 {
            data: val,
            default: val,
            help: help.to_owned(),
            cb: Some(cb),
        };
        self.data.insert(key.to_owned(), EnvMetaData::I64(meta));
        if !self.exec_i64_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        Ok(())
    }

    pub fn add_i64(&mut self, key: &str, val: i64, help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvI64 {
            data: val,
            default: val,
            help: help.to_owned(),
            cb: None,
        };
        self.data.insert(key.to_owned(), EnvMetaData::I64(meta));
        Ok(())
    }

    pub fn get_i64(&self, key: &str) -> Result<i64, EnvErr> {
        let Some(meta) = self.data.get(key) else {
            return Err(EnvErr::NotFound);
        };
        match meta.as_i64() {
            Some(s) => Ok(s.data),
            None => Err(EnvErr::DifferentType),
        }
    }

    pub fn set_i64(&mut self, key: &str, value: i64, data: &mut T) -> Result<(), EnvErr> {
        let Some(meta) = self.data.get_mut(key) else {
            return Err(EnvErr::NotFound);
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
            Ok(())
        } else {
            Err(EnvErr::DifferentType)
        }
    }

    #[must_use]
    pub fn is_i64(&self, key: &str) -> bool {
        let Some(meta) = self.data.get(key) else {
            return false;
        };
        meta.as_i64().is_some()
    }

    fn exec_bool_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data[key].as_bool().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, data);
        }
        true
    }

    pub fn add_bool_with_cb(
        &mut self,
        key: &str,
        val: bool,
        help: &str,
        data: &mut T,
        cb: BoolFn<T>,
    ) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvBool {
            data: val,
            default: val,
            help: help.to_owned(),
            cb: Some(cb),
        };
        self.data.insert(key.to_owned(), EnvMetaData::Bool(meta));
        if !self.exec_bool_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        Ok(())
    }

    pub fn add_bool(&mut self, key: &str, val: bool, help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvBool {
            data: val,
            default: val,
            help: help.to_owned(),
            cb: None,
        };
        self.data.insert(key.to_owned(), EnvMetaData::Bool(meta));
        Ok(())
    }

    pub fn get_bool(&self, key: &str) -> Result<bool, EnvErr> {
        let Some(meta) = self.data.get(key) else {
            return Err(EnvErr::NotFound);
        };
        match meta.as_bool() {
            Some(s) => Ok(s.data),
            None => Err(EnvErr::DifferentType),
        }
    }

    pub fn set_bool(&mut self, key: &str, value: bool, data: &mut T) -> Result<(), EnvErr> {
        let Some(meta) = self.data.get_mut(key) else {
            return Err(EnvErr::NotFound);
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
            Ok(())
        } else {
            Err(EnvErr::DifferentType)
        }
    }

    #[must_use]
    pub fn is_bool(&self, key: &str) -> bool {
        let Some(meta) = self.data.get(key) else {
            return false;
        };
        meta.as_bool().is_some()
    }

    fn exec_color_cb(&self, key: &str, data: &mut T) -> bool {
        let meta = self.data[key].as_color().unwrap();
        if let Some(cb) = meta.cb {
            return cb(key, meta.data, self, data);
        }
        true
    }

    pub fn add_color_with_cb(
        &mut self,
        key: &str,
        val: (u8, u8, u8),
        help: &str,
        data: &mut T,
        cb: ColorFn<T>,
    ) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvColor {
            data: val,
            default: val,
            help: help.to_owned(),
            cb: Some(cb),
        };
        self.data.insert(key.to_owned(), EnvMetaData::Color(meta));
        if !self.exec_color_cb(key, data) {
            self.data.remove(key).unwrap();
            return Err(EnvErr::CbFailed);
        }
        Ok(())
    }

    pub fn add_color(&mut self, key: &str, val: (u8, u8, u8), help: &str) -> Result<(), EnvErr> {
        if self.data.contains_key(key) {
            return Err(EnvErr::AlreadyExist);
        }
        let meta = EnvColor {
            data: val,
            default: val,
            help: help.to_owned(),
            cb: None,
        };
        self.data.insert(key.to_owned(), EnvMetaData::Color(meta));
        Ok(())
    }

    pub fn get_color(&self, key: &str) -> Result<(u8, u8, u8), EnvErr> {
        let Some(meta) = self.data.get(key) else {
            return Err(EnvErr::NotFound);
        };
        match meta.as_color() {
            Some(s) => Ok(s.data),
            None => Err(EnvErr::DifferentType),
        }
    }

    pub fn set_color(
        &mut self,
        key: &str,
        value: (u8, u8, u8),
        data: &mut T,
    ) -> Result<(), EnvErr> {
        let Some(meta) = self.data.get_mut(key) else {
            return Err(EnvErr::NotFound);
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
            Ok(())
        } else {
            Err(EnvErr::DifferentType)
        }
    }

    #[must_use]
    pub fn is_color(&self, key: &str) -> bool {
        let Some(meta) = self.data.get(key) else {
            return false;
        };
        meta.as_color().is_some()
    }

    pub fn reset(&mut self, key: &str, data: &mut T) -> Result<(), EnvErr> {
        let Some(meta) = self.data.get_mut(key) else {
            return Err(EnvErr::NotFound);
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
        Ok(())
    }
    #[must_use]
    pub fn get(&self, key: &str) -> Option<EnvData<'_>> {
        let meta = self.data.get(key)?;
        match meta {
            EnvMetaData::Bool(b) => Some(EnvData::Bool(b.data)),
            EnvMetaData::I64(i) => Some(EnvData::I64(i.data)),
            EnvMetaData::U64(u) => Some(EnvData::U64(u.data)),
            EnvMetaData::Str(s) => Some(EnvData::Str(&s.data)),
            EnvMetaData::Color(c) => Some(EnvData::Color(c.data.0, c.data.1, c.data.2)),
        }
    }
    #[must_use]
    pub fn get_help(&self, key: &str) -> Option<&str> {
        let meta = self.data.get(key)?;
        match meta {
            EnvMetaData::Bool(b) => Some(&b.help),
            EnvMetaData::I64(i) => Some(&i.help),
            EnvMetaData::U64(u) => Some(&u.help),
            EnvMetaData::Str(s) => Some(&s.help),
            EnvMetaData::Color(c) => Some(&c.help),
        }
    }
    #[must_use]
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = (&str, EnvData<'_>)> + 'a> {
        Box::new(
            self.data
                .iter()
                .map(|(k, v)| (k.as_str(), EnvData::from(v))),
        )
    }
}

impl<'a, T> IntoIterator for &'a Environment<T> {
    type IntoIter =
        Box<(dyn Iterator<Item = (&'a str, EnvData<'a>)> + 'a)>;
    type Item = (&'a str, EnvData<'a>);
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test_environment {
    use super::*;
    fn even_str(_: &str, value: &str, _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        value.len() % 2 == 0
    }
    fn even_u64(_: &str, value: u64, _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        value % 2 == 0
    }
    fn negative_i64(_: &str, value: i64, _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        value < 0
    }
    fn always_false(_: &str, value: bool, _: &Environment<Option<()>>, _: &mut Option<()>) -> bool {
        !value
    }
    fn grayscale(
        _: &str,
        color: (u8, u8, u8),
        _: &Environment<Option<()>>,
        _: &mut Option<()>,
    ) -> bool {
        let (r, g, b) = color;
        r == g && g == b
    }
    fn prep_env() -> Environment<Option<()>> {
        let mut data = None;
        let mut env = Environment::new();
        env.add_str("s1", "value1", "First String").unwrap();
        env.add_str_with_cb("s2", "value2", "Second String", &mut data, even_str)
            .unwrap();
        env.add_u64("u1", 1, "First U64").unwrap();
        env.add_u64_with_cb("u2", 2, "Second U64", &mut data, even_u64)
            .unwrap();
        env.add_i64("i1", 1, "First I64").unwrap();
        env.add_i64_with_cb("i2", -1, "Second I64", &mut data, negative_i64)
            .unwrap();
        env.add_bool("b1", true, "First Bool").unwrap();
        env.add_bool_with_cb("b2", false, "Second Bool", &mut data, always_false)
            .unwrap();
        env.add_color("c1", (31, 33, 37), "First Color").unwrap();
        env.add_color_with_cb("c2", (50, 50, 50), "Second Color", &mut data, grayscale)
            .unwrap();
        env
    }
    #[test]
    fn test_str() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(
            env.add_str_with_cb("s03", "value02", "", &mut data, even_str)
                .err()
                .unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(
            env.add_str("s1", "v3", "").err().unwrap(),
            EnvErr::AlreadyExist
        );
        assert_eq!(
            env.add_str_with_cb("s1", "value1", "", &mut data, even_str)
                .err()
                .unwrap(),
            EnvErr::AlreadyExist
        );
        assert!(env.is_str("s1"));
        assert!(!env.is_str("u1"));
        assert!(!env.is_str("s3"));
        assert_eq!(env.get_str("s1").unwrap(), "value1");
        assert_eq!(env.get_str("s2").unwrap(), "value2");
        assert_eq!(env.get_str("s3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_str("u1").err().unwrap(), EnvErr::DifferentType);
        env.set_str("s1", "newvalue1", &mut data).unwrap();
        assert_eq!(env.get_str("s1").unwrap(), "newvalue1");
        env.set_str("s2", "newvalue02", &mut data).unwrap();
        assert_eq!(env.get_str("s2").unwrap(), "newvalue02");
        assert_eq!(
            env.set_str("s2", "tmp", &mut data).err().unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(env.get_str("s2").unwrap(), "newvalue02");
        assert_eq!(
            env.set_str("s3", "tmp", &mut data).err().unwrap(),
            EnvErr::NotFound
        );
        assert_eq!(
            env.set_str("u1", "tmp", &mut data).err().unwrap(),
            EnvErr::DifferentType
        );
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
        assert_eq!(
            env.add_u64_with_cb("u3", 3, "", &mut data, even_u64)
                .err()
                .unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(
            env.add_u64("u2", 5, "").err().unwrap(),
            EnvErr::AlreadyExist
        );
        assert_eq!(
            env.add_u64_with_cb("s1", 4, "", &mut data, even_u64)
                .err()
                .unwrap(),
            EnvErr::AlreadyExist
        );
        assert!(env.is_u64("u1"));
        assert!(!env.is_u64("s1"));
        assert!(!env.is_u64("u3"));
        assert_eq!(env.get_u64("u1").unwrap(), 1);
        assert_eq!(env.get_u64("u2").unwrap(), 2);
        assert_eq!(env.get_u64("u3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_u64("s1").err().unwrap(), EnvErr::DifferentType);
        env.set_u64("u1", 8, &mut data).unwrap();
        assert_eq!(env.get_u64("u1").unwrap(), 8);
        env.set_u64("u2", 4, &mut data).unwrap();
        assert_eq!(env.get_u64("u2").unwrap(), 4);
        assert_eq!(
            env.set_u64("u2", 7, &mut data).err().unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(env.get_u64("u2").unwrap(), 4);
        assert_eq!(
            env.set_u64("u3", 5, &mut data).err().unwrap(),
            EnvErr::NotFound
        );
        assert_eq!(
            env.set_u64("s1", 3, &mut data).err().unwrap(),
            EnvErr::DifferentType
        );
        assert_eq!(env.get("u1").unwrap(), EnvData::U64(8));
        env.reset("u1", &mut data).unwrap();
        assert_eq!(env.get_u64("u1").unwrap(), 1);
        assert_eq!(env.get_help("u1").unwrap(), "First U64");
    }
    #[test]
    fn test_i64() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(
            env.add_i64_with_cb("i3", 3, "", &mut data, negative_i64)
                .err()
                .unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(
            env.add_i64("i2", 5, "").err().unwrap(),
            EnvErr::AlreadyExist
        );
        assert_eq!(
            env.add_i64_with_cb("s1", 4, "", &mut data, negative_i64)
                .err()
                .unwrap(),
            EnvErr::AlreadyExist
        );
        assert!(env.is_i64("i1"));
        assert!(!env.is_i64("u1"));
        assert!(!env.is_i64("i3"));
        assert_eq!(env.get_i64("i1").unwrap(), 1);
        assert_eq!(env.get_i64("i2").unwrap(), -1);
        assert_eq!(env.get_i64("u3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_i64("s1").err().unwrap(), EnvErr::DifferentType);
        env.set_i64("i1", 8, &mut data).unwrap();
        assert_eq!(env.get_i64("i1").unwrap(), 8);
        env.set_i64("i2", -4, &mut data).unwrap();
        assert_eq!(env.get_i64("i2").unwrap(), -4);
        assert_eq!(
            env.set_i64("i2", 7, &mut data).err().unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(env.get_i64("i2").unwrap(), -4);
        assert_eq!(
            env.set_i64("i3", 5, &mut data).err().unwrap(),
            EnvErr::NotFound
        );
        assert_eq!(
            env.set_i64("s1", 3, &mut data).err().unwrap(),
            EnvErr::DifferentType
        );
        assert_eq!(env.get("i1").unwrap(), EnvData::I64(8));
        env.reset("i1", &mut data).unwrap();
        assert_eq!(env.get_i64("i1").unwrap(), 1);
        assert_eq!(env.get_help("i1").unwrap(), "First I64");
    }
    #[test]
    fn test_bool() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(
            env.add_bool_with_cb("b3", true, "", &mut data, always_false)
                .err()
                .unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(
            env.add_bool("b2", true, "").err().unwrap(),
            EnvErr::AlreadyExist
        );
        assert_eq!(
            env.add_bool_with_cb("b1", false, "", &mut data, always_false)
                .err()
                .unwrap(),
            EnvErr::AlreadyExist
        );
        assert!(env.is_bool("b1"));
        assert!(!env.is_bool("u1"));
        assert!(!env.is_bool("b3"));
        assert!(env.get_bool("b1").unwrap());
        assert!(!env.get_bool("b2").unwrap());
        assert_eq!(env.get_bool("u3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_bool("s1").err().unwrap(), EnvErr::DifferentType);
        env.set_bool("b1", false, &mut data).unwrap();
        assert!(!env.get_bool("b1").unwrap());
        assert_eq!(
            env.set_bool("b2", true, &mut data).err().unwrap(),
            EnvErr::CbFailed
        );
        assert!(!env.get_bool("b2").unwrap());
        assert_eq!(
            env.set_bool("b3", true, &mut data).err().unwrap(),
            EnvErr::NotFound
        );
        assert_eq!(
            env.set_bool("s1", false, &mut data).err().unwrap(),
            EnvErr::DifferentType
        );
        assert_eq!(env.get("b1").unwrap(), EnvData::Bool(false));
        env.reset("b1", &mut data).unwrap();
        assert!(env.get_bool("b1").unwrap());
        assert_eq!(env.get_help("b1").unwrap(), "First Bool");
    }

    #[test]
    fn test_color() {
        let mut env = prep_env();
        let mut data = None;
        assert_eq!(
            env.add_color_with_cb("c3", (50, 60, 70), "", &mut data, grayscale)
                .err()
                .unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(
            env.add_color("c2", (20, 30, 40), "").err().unwrap(),
            EnvErr::AlreadyExist
        );
        assert_eq!(
            env.add_color_with_cb("c1", (100, 100, 100), "", &mut data, grayscale)
                .err()
                .unwrap(),
            EnvErr::AlreadyExist
        );
        assert!(env.is_color("c1"));
        assert!(!env.is_color("u1"));
        assert!(!env.is_color("c3"));
        /**/
        assert_eq!(env.get_color("c1").unwrap(), (31, 33, 37));
        assert_eq!(env.get_color("c2").unwrap(), (50, 50, 50));
        assert_eq!(env.get_color("c3").err().unwrap(), EnvErr::NotFound);
        assert_eq!(env.get_color("s1").err().unwrap(), EnvErr::DifferentType);
        env.set_color("c1", (20, 40, 60), &mut data).unwrap();
        assert_eq!(env.get_color("c1").unwrap(), (20, 40, 60));
        assert_eq!(
            env.set_color("c2", (5, 10, 15), &mut data).err().unwrap(),
            EnvErr::CbFailed
        );
        assert_eq!(env.get_color("c2").unwrap(), (50, 50, 50));
        assert_eq!(
            env.set_color("c3", (50, 50, 50), &mut data).err().unwrap(),
            EnvErr::NotFound
        );
        assert_eq!(
            env.set_color("s1", (50, 50, 50), &mut data).err().unwrap(),
            EnvErr::DifferentType
        );
        assert_eq!(env.get("c1").unwrap(), EnvData::Color(20, 40, 60));
        env.reset("c1", &mut data).unwrap();
        assert_eq!(env.get_color("c1").unwrap(), (31, 33, 37));
        assert_eq!(env.get_help("c1").unwrap(), "First Color");
    }
}
