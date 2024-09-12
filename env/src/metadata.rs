//! Data types handling handling for renv.

use super::environment::Environment;

pub type StrFn<T> = fn(&str, &str, &Environment<T>, &mut T) -> bool;
pub type U64Fn<T> = fn(&str, u64, &Environment<T>, &mut T) -> bool;
pub type I64Fn<T> = fn(&str, i64, &Environment<T>, &mut T) -> bool;
pub type BoolFn<T> = fn(&str, bool, &Environment<T>, &mut T) -> bool;
pub type ColorFn<T> = fn(&str, (u8, u8, u8), &Environment<T>, &mut T) -> bool;

pub(crate) struct EnvStr<T> {
    pub(crate) data: String,
    pub(crate) default: String,
    pub(crate) help: String,
    pub(crate) cb: Option<StrFn<T>>,
}

pub(crate) struct EnvU64<T> {
    pub(crate) data: u64,
    pub(crate) default: u64,
    pub(crate) help: String,
    pub(crate) cb: Option<U64Fn<T>>,
}

pub(crate) struct EnvI64<T> {
    pub(crate) data: i64,
    pub(crate) default: i64,
    pub(crate) help: String,
    pub(crate) cb: Option<I64Fn<T>>,
}

pub(crate) struct EnvBool<T> {
    pub(crate) data: bool,
    pub(crate) default: bool,
    pub(crate) help: String,
    pub(crate) cb: Option<BoolFn<T>>,
}

pub(crate) struct EnvColor<T> {
    pub(crate) data: (u8, u8, u8),
    pub(crate) default: (u8, u8, u8),
    pub(crate) help: String,
    pub(crate) cb: Option<ColorFn<T>>,
}

pub(crate) enum EnvMetaData<T> {
    Str(EnvStr<T>),
    U64(EnvU64<T>),
    I64(EnvI64<T>),
    Bool(EnvBool<T>),
    Color(EnvColor<T>),
}

impl<T> EnvMetaData<T> {
    pub(crate) fn as_str(&self) -> Option<&EnvStr<T>> {
        if let EnvMetaData::Str(s) = self {
            return Some(s);
        }
        None
    }
    pub(crate) fn as_u64(&self) -> Option<&EnvU64<T>> {
        if let EnvMetaData::U64(u) = self {
            return Some(u);
        }
        None
    }
    pub(crate) fn as_i64(&self) -> Option<&EnvI64<T>> {
        if let EnvMetaData::I64(i) = self {
            return Some(i);
        }
        None
    }
    pub(crate) fn as_bool(&self) -> Option<&EnvBool<T>> {
        if let EnvMetaData::Bool(b) = self {
            return Some(b);
        }
        None
    }
    pub(crate) fn as_color(&self) -> Option<&EnvColor<T>> {
        if let EnvMetaData::Color(c) = self {
            return Some(c);
        }
        None
    }
    pub(crate) fn mut_str(&mut self) -> Option<&mut EnvStr<T>> {
        if let EnvMetaData::Str(s) = self {
            return Some(s);
        }
        None
    }
    pub(crate) fn mut_u64(&mut self) -> Option<&mut EnvU64<T>> {
        if let EnvMetaData::U64(u) = self {
            return Some(u);
        }
        None
    }
    pub(crate) fn mut_i64(&mut self) -> Option<&mut EnvI64<T>> {
        if let EnvMetaData::I64(i) = self {
            return Some(i);
        }
        None
    }
    pub(crate) fn mut_bool(&mut self) -> Option<&mut EnvBool<T>> {
        if let EnvMetaData::Bool(b) = self {
            return Some(b);
        }
        None
    }
    pub(crate) fn mut_color(&mut self) -> Option<&mut EnvColor<T>> {
        if let EnvMetaData::Color(c) = self {
            return Some(c);
        }
        None
    }
}
