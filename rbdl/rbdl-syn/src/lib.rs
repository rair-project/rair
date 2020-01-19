/*
 * rbdl-syn: rair binary dexcriptor language extension for syn crate
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
extern crate syn;
mod attrs;
mod enums;
mod fields;
mod types;
mod value;
mod vec;

pub use attrs::*;
pub use enums::*;
pub use fields::*;
use syn::parse::{Parse, ParseStream};
use syn::token::{Colon, Struct};
use syn::{Ident, Result, Token};
pub use types::*;
pub use value::*;
pub use vec::*;

#[derive(Debug)]
pub struct RBDLFile {
    pub items: Vec<RBDLItem>,
}

impl Parse for RBDLFile {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RBDLFile {
            items: {
                let mut items = Vec::new();
                while !input.is_empty() {
                    items.push(input.parse()?);
                }
                items
            },
        })
    }
}

#[derive(Debug)]
pub enum RBDLItem {
    Struct(RBDLStruct),
    Enum(RBDLEnum),
}

impl Parse for RBDLItem {
    fn parse(input: ParseStream) -> Result<Self> {
        let ahead = input.fork();
        ahead.call::<Option<Attributes>>(Attributes::parse_outer)?;
        ahead.parse::<Ident>()?;
        ahead.parse::<Colon>()?;
        let lookahead = ahead.lookahead1();
        if lookahead.peek(Token!(struct)) {
            Ok(RBDLItem::Struct(input.parse()?))
        } else if lookahead.peek(Token!(enum)) {
            Ok(RBDLItem::Enum(input.parse()?))
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug)]
pub struct RBDLStruct {
    pub attrs: Option<Attributes>,
    pub ident: Ident,
    pub colon_token: Colon,
    pub struct_token: Struct,
    pub fields: RBDLFields,
}

impl Parse for RBDLStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RBDLStruct {
            attrs: input.call(Attributes::parse_outer)?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            struct_token: input.parse()?,
            fields: input.parse()?,
        })
    }
}
