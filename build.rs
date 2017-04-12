/*
 *  build.rs -- Building Script for rair
 *  Copyright (C) 2017  Ahmed Abd El Mawgood
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("version_info.rs");
    let mut f = File::create(&dest_path).unwrap();
    let version = env!("CARGO_PKG_VERSION");
    let out = format!(
        "pub static TARGET:&'static str = \"{}\";
                        pub static VERSION:&'static str = \"{}\";
                        ",
        target, version
    );
    f.write_all(out.as_bytes()).unwrap();
}
