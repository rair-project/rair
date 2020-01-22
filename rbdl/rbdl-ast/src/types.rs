/*
 * types.rs: Comparable Type representation in for rbdl.
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

use rbdl_syn::RBDLType;
use syn::Ident;

#[derive(Debug, PartialEq)]
pub struct AstType {
    pub ty: Ident,
    pub args: Vec<AstType>,
}
impl From<RBDLType> for AstType {
    fn from(parse_tree: RBDLType) -> AstType {
        AstType {
            ty: parse_tree.ident,
            args: match parse_tree.arguments {
                None => Vec::new(),
                Some(types) => types.args.into_iter().map(|t| t.into()).collect(),
            },
        }
    }
}
