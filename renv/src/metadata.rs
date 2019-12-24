/*
 * metadata.rs: Data types handling handling for renv.
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
use super::environment::Environment;

pub type StrFn<T> = fn(&str, &str, &Environment<T>, &mut T) -> bool;
pub type U64Fn<T> = fn(&str, u64, &Environment<T>, &mut T) -> bool;
pub type I64Fn<T> = fn(&str, i64, &Environment<T>, &mut T) -> bool;
pub type BoolFn<T> = fn(&str, bool, &Environment<T>, &mut T) -> bool;

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

pub(crate) enum EnvMetaData<T> {
    Str(EnvStr<T>),
    U64(EnvU64<T>),
    I64(EnvI64<T>),
    Bool(EnvBool<T>),
}

impl<T> EnvMetaData<T> {
    pub(crate) fn as_str(&self) -> Option<&EnvStr<T>> {
        if let EnvMetaData::Str(s) = self {
            return Some(s);
        }
        return None;
    }
    pub(crate) fn as_u64(&self) -> Option<&EnvU64<T>> {
        if let EnvMetaData::U64(u) = self {
            return Some(u);
        }
        return None;
    }
    pub(crate) fn as_i64(&self) -> Option<&EnvI64<T>> {
        if let EnvMetaData::I64(i) = self {
            return Some(i);
        }
        return None;
    }
    pub(crate) fn as_bool(&self) -> Option<&EnvBool<T>> {
        if let EnvMetaData::Bool(b) = self {
            return Some(b);
        }
        return None;
    }

    pub(crate) fn mut_str(&mut self) -> Option<&mut EnvStr<T>> {
        if let EnvMetaData::Str(s) = self {
            return Some(s);
        }
        return None;
    }
    pub(crate) fn mut_u64(&mut self) -> Option<&mut EnvU64<T>> {
        if let EnvMetaData::U64(u) = self {
            return Some(u);
        }
        return None;
    }
    pub(crate) fn mut_i64(&mut self) -> Option<&mut EnvI64<T>> {
        if let EnvMetaData::I64(i) = self {
            return Some(i);
        }
        return None;
    }
    pub(crate) fn mut_bool(&mut self) -> Option<&mut EnvBool<T>> {
        if let EnvMetaData::Bool(b) = self {
            return Some(b);
        }
        return None;
    }
}
