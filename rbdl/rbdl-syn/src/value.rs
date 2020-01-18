/*
 * value.rs: RBDL Attributes value implementation
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

use super::vec::*;
use syn::parse::{Parse, ParseStream};
use syn::token::Bracket;
use syn::{Ident, Lit, LitBool, LitByte, LitByteStr, LitChar, LitFloat, LitInt, LitStr, Result};

#[derive(Debug)]
pub enum RBDLValue {
    Str(LitStr),
    ByteStr(LitByteStr),
    Byte(LitByte),
    Char(LitChar),
    Int(LitInt),
    Float(LitFloat),
    Bool(LitBool),
    Ident(Ident),
    Vec(RBDLVec<RBDLValue>),
}

impl RBDLValue {
    fn parse_vec(input: ParseStream) -> Result<Self> {
        let vec = input.parse::<RBDLVec<RBDLValue>>()?;
        Ok(RBDLValue::Vec(vec))
    }

    fn parse_lit(input: ParseStream) -> Result<Self> {
        if input.peek(Lit) {
            let lit = match input.parse::<Lit>()? {
                Lit::Str(s) => RBDLValue::Str(s),
                Lit::ByteStr(bs) => RBDLValue::ByteStr(bs),
                Lit::Byte(b) => RBDLValue::Byte(b),
                Lit::Char(c) => RBDLValue::Char(c),
                Lit::Int(i) => RBDLValue::Int(i),
                Lit::Float(f) => RBDLValue::Float(f),
                Lit::Bool(b) => RBDLValue::Bool(b),
                _ => panic!("Unknown literal type"),
            };
            return Ok(lit);
        } else if input.peek(Ident) {
            return Ok(RBDLValue::Ident(input.parse::<Ident>()?));
        } else {
            panic!("Expected Idenitifier or Literal.");
        }
    }
}
impl Parse for RBDLValue {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Bracket) {
            RBDLValue::parse_vec(input)
        } else {
            RBDLValue::parse_lit(input)
        }
    }
}
