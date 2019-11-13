/*
 * commands.rs: Data Structure for holding rair commands.
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

use helper::*;
use rtrees::bktree::SpellTree;
use std::collections::BTreeMap; // for suffex search

#[derive(Default)]
pub struct Commands {
    suggestions: SpellTree<()>,
    search: BTreeMap<&'static str, Box<dyn Cmd>>,
}

impl Commands {
    // Returns false if the command with the same name exists
    pub fn add_command(&mut self, command_name: &'static str, functionality: Box<dyn Cmd>) -> bool {
        // first check that command_name doesn't exist
        if self.search.contains_key(command_name) {
            return false;
        } else {
            self.suggestions.insert(command_name.to_string(), ());
            self.search.insert(command_name, functionality);
            return true;
        }
    }

    pub fn find(&self, command: &str) -> Option<&dyn Cmd> {
        return self.search.get(command).map(|cmd| &**cmd);
    }
    pub fn find_mut(&mut self, command: &str) -> Option<&mut dyn Cmd> {
        return self.search.get_mut(command).map(|cmd| &mut **cmd as &mut dyn Cmd);
    }
    pub fn suggest(&self, command: &str, tolerance: u64) -> Vec<&String> {
        return self.suggestions.find(&command.to_string(), tolerance).1;
    }
    pub fn prefix<'a>(&'a self, command: &'a str) -> Vec<&&str> {
        return self.search.range(command..).take_while(|(k, _)| k.starts_with(command)).map(|(k, _)| k).collect();
    }
}
