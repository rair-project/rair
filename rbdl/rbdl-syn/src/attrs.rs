/*
 * attrs.rs: RBDL Attributes implementation
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
use super::value::*;
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Bracket, Comma, Eq, Pound};
use syn::{bracketed, Ident, Result, Token};

#[derive(Debug)]
pub struct Attributes {
    pub pound_token: Pound,
    pub bracket_token: Bracket,
    pub attrs: Punctuated<Attribute, Comma>,
}

impl Attributes {
    pub fn parse_outer(input: ParseStream) -> Result<Option<Self>> {
        if input.peek(Token!(#)) {
            Attributes::parse(input).map(|ok| Some(ok))
        } else {
            Ok(None)
        }
    }
}

impl Parse for Attributes {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Attributes {
            pound_token: input.parse()?,
            bracket_token: bracketed!(content in input),
            attrs: content.parse_terminated(Attribute::parse)?,
        })
    }
}

#[derive(Debug)]
pub enum Attribute {
    Valued(ValuedAttribute),
    Unvalued(UnvaluedAttribute),
}

impl Parse for Attribute {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek2(Token!(=)) {
            input.parse().map(Attribute::Valued)
        } else {
            input.parse().map(Attribute::Unvalued)
        }
    }
}

#[derive(Debug)]
pub struct UnvaluedAttribute {
    pub ident: Ident,
}

impl Parse for UnvaluedAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(UnvaluedAttribute { ident: input.parse()? })
    }
}

#[derive(Debug)]
pub struct ValuedAttribute {
    pub ident: Ident,
    pub equal_token: Eq,
    pub value: RBDLValue,
}

impl Parse for ValuedAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ValuedAttribute {
            ident: input.call(Ident::parse_any)?,
            equal_token: input.parse()?,
            value: input.parse()?,
        })
    }
}
