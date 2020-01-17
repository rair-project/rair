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
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Colon, Comma, Enum, Eq, Pound, Struct};
use syn::{braced, bracketed, Ident, Lit, LitBool, LitByte, LitByteStr, LitChar, LitFloat, LitInt, LitStr, Result, Token, Type};

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

#[derive(Debug)]
pub struct RBDLEnum {
    pub attrs: Option<Attributes>,
    pub ident: Ident,
    pub colon_token: Colon,
    pub enum_token: Enum,
    pub fields: RBDLFields,
}

impl Parse for RBDLEnum {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RBDLEnum {
            attrs: input.call(Attributes::parse_outer)?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            enum_token: input.parse()?,
            fields: input.parse()?,
        })
    }
}

#[derive(Debug)]
pub struct RBDLFields {
    pub brace_token: Brace,
    pub named: Punctuated<RBDLField, Comma>,
}

impl Parse for RBDLFields {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(RBDLFields {
            brace_token: braced!(content in input),
            named: content.parse_terminated(RBDLField::parse)?,
        })
    }
}

#[derive(Debug)]
pub struct RBDLField {
    pub attrs: Option<Attributes>,
    pub ident: Ident,
    pub colon_token: Colon,
    pub ty: Type,
}

impl Parse for RBDLField {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(RBDLField {
            attrs: input.call(Attributes::parse_outer)?,
            ident: input.parse()?,
            colon_token: input.parse()?,
            ty: input.parse()?,
        })
    }
}

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
pub struct ValuedAttribute {
    pub ident: Ident,
    pub equal_token: Eq,
    pub value: RBDLLit,
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

#[derive(Debug)]
pub enum RBDLLit {
    Str(LitStr),
    ByteStr(LitByteStr),
    Byte(LitByte),
    Char(LitChar),
    Int(LitInt),
    Float(LitFloat),
    Bool(LitBool),
    Ident(Ident),
    StrVec(LitVec<LitStr>),
    ByteStrVec(LitVec<LitByteStr>),
    ByteVec(LitVec<LitByte>),
    CharVec(LitVec<LitChar>),
    IntVec(LitVec<LitInt>),
    FloatVec(LitVec<LitFloat>),
    BoolVec(LitVec<LitBool>),
    IdentVec(LitVec<Ident>)
}

impl RBDLLit {
    fn parse_vec(input: ParseStream) -> Result<Self> {
        let ahead = input.fork();
        let content;
        bracketed!(content in ahead);
        let vec;
        if content.peek(Lit) {
            vec = match content.parse::<Lit>()? {
                Lit::Str(_) => RBDLLit::StrVec(input.parse::<LitVec<LitStr>>()?),
                Lit::ByteStr(_) => RBDLLit::ByteStrVec(input.parse::<LitVec<LitByteStr>>()?),
                Lit::Byte(_) => RBDLLit::ByteVec(input.parse::<LitVec<LitByte>>()?),
                Lit::Char(_) => RBDLLit::CharVec(input.parse::<LitVec<LitChar>>()?),
                Lit::Int(_) => RBDLLit::IntVec(input.parse::<LitVec<LitInt>>()?),
                Lit::Float(_) => RBDLLit::FloatVec(input.parse::<LitVec<LitFloat>>()?),
                Lit::Bool(_) => RBDLLit::BoolVec(input.parse::<LitVec<LitBool>>()?),
                _ => panic!("Unknown literal type"),
            };
        } else if content.peek(Ident) {
            vec = RBDLLit::IdentVec(input.parse::<LitVec<Ident>>()?);
        } else {
            panic!("Expected Idenitifier or Literal.");
        }
        Ok(vec)
    }
    fn parse_lit(input: ParseStream) -> Result<Self> {
        if input.peek(Lit) {
            let lit = match input.parse::<Lit>()? {
                Lit::Str(s) => RBDLLit::Str(s),
                Lit::ByteStr(bs) => RBDLLit::ByteStr(bs),
                Lit::Byte(b) => RBDLLit::Byte(b),
                Lit::Char(c) => RBDLLit::Char(c),
                Lit::Int(i) => RBDLLit::Int(i),
                Lit::Float(f) => RBDLLit::Float(f),
                Lit::Bool(b) => RBDLLit::Bool(b),
                _ => panic!("Unknown literal type"),
            };
            return Ok(lit);
        } else if input.peek(Ident) {
            return Ok(RBDLLit::Ident(input.parse::<Ident>()?));
        } else {
            panic!("Expected Idenitifier or Literal.");
        }
    }
}
impl Parse for RBDLLit {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Bracket) {
            RBDLLit::parse_vec(input)
        } else {
            RBDLLit::parse_lit(input)
        }
    }
}


#[derive(Debug)]
pub struct LitVec<T> {
    pub bracket_token: Bracket,
    pub attrs: Punctuated<T, Comma>,
}

impl<T> Parse for LitVec<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(LitVec {
            bracket_token: bracketed!(content in input),
            attrs: content.parse_terminated(T::parse)?,
        })
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
