extern crate proc_macro;
/*
 * rbdl-derive: rair binary descriptor language derive macros.
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
extern crate rbdl_syn;
extern crate syn;

// extern crate quote;
use proc_macro::TokenStream;
use rbdl_syn::*;
use std::fs::File as FsFile;
use std::io::prelude::*;
use std::path::PathBuf;
use syn::{parse_macro_input, LitStr};

#[proc_macro]
pub fn rbdl_include(input: TokenStream) -> TokenStream {
    let definition_file = parse_macro_input!(input as LitStr);
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.push(definition_file.value());
    let mut file = match FsFile::open(&path) {
        Ok(file) => file,
        Err(e) => panic!(format!("{}: {}", &definition_file.value(), e)),
    };
    let mut definitions = String::new();
    if let Err(e) = file.read_to_string(&mut definitions) {
        panic!(format!("{}: {}", &definition_file.value(), e));
    }
    let stream: proc_macro::TokenStream = definitions.parse().unwrap();
    return rbdl_inline(stream);
}

#[proc_macro]
pub fn rbdl_inline(input: TokenStream) -> TokenStream {
    let _parse_tree = parse_macro_input!(input as RBDLFile);
    //println!("{:#?}", parse_tree);
    return TokenStream::new();
}
