/*
 * table.rs: Thin wrappr around HashMap<Ident, AstAttrValue>.
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
use super::AstAttrValue;
use rbdl_syn::{Attribute, Attributes};
use std::collections::HashMap;
use std::convert::TryFrom;
use syn::{Error, Ident, Result};

pub struct Table(HashMap<Ident, AstAttrValue>);

impl Table {
    pub fn unwrap(self) -> HashMap<Ident, AstAttrValue> {
        self.0
    }
}
impl TryFrom<Attributes> for Table {
    type Error = Error;

    fn try_from(parse_tree: Attributes) -> Result<Table> {
        let mut tbl = HashMap::new();
        let mut errs: Option<Error> = None;
        for attr in parse_tree.attrs {
            let (k, v): (Ident, AstAttrValue) = match attr {
                Attribute::Valued(v) => (v.ident, v.value.into()),
                Attribute::Unvalued(u) => (u.ident, AstAttrValue::None),
            };
            if tbl.contains_key(&k) {
                let err = Error::new(k.span(), format!("Attribute `{}` has been already set before", k));
                if errs.is_some() {
                    errs.as_mut().unwrap().combine(err);
                } else {
                    errs = Some(err);
                }
            } else {
                tbl.insert(k, v);
            }
        }
        match errs {
            Some(e) => Err(e),
            None => Ok(Table(tbl)),
        }
    }
}
