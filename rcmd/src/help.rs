/*
 * help.rs: Help part of AST.
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

use grammar::Rule;
use pest::iterators::Pair;

#[derive(Default, Debug)]
pub struct HelpCmd {
    pub command: String,
}

impl HelpCmd {
    pub(crate) fn parse_help(root: Pair<Rule>) -> Self {
        assert_eq!(root.as_rule(), Rule::HelpLine);
        HelpCmd {
            command: root.into_inner().next().unwrap().as_str().to_owned(),
        }
    }
}
