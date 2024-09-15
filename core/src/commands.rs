//! Data Structure for holding rair commands.

use std::collections::HashSet;

use crate::{cmd::Cmd, helper::MRc};
use alloc::collections::BTreeMap;
use rair_trees::bktree::SpellTree; // for suffex search

#[derive(Default)]
pub struct Commands {
    suggestions: SpellTree<()>,
    search: BTreeMap<&'static str, MRc<dyn Cmd + Sync + Send>>,
}

impl Commands {
    // Returns false if the command with the same name exists
    pub fn add_command(
        &mut self,
        command_name: &'static str,
        functionality: MRc<dyn Cmd + Sync + Send>,
    ) -> bool {
        // first check that command_name doesn't exist
        if self.search.contains_key(command_name) {
            false
        } else {
            self.suggestions.insert(command_name.to_owned(), ());
            self.search.insert(command_name, functionality);
            true
        }
    }

    #[must_use]
    pub fn find(&self, command: &str) -> Option<MRc<dyn Cmd + Sync + Send>> {
        self.search.get(command).cloned()
    }
    #[must_use]
    pub fn suggest(&self, command: &str, tolerance: u64) -> Vec<&String> {
        self.suggestions.find(&command.to_owned(), tolerance).1
    }
    #[must_use]
    pub fn prefix<'a>(&'a self, command: &'a str) -> Vec<&&str> {
        self.search
            .range(command..)
            .take_while(|(k, _)| k.starts_with(command))
            .map(|(k, _)| k)
            .collect()
    }
    /// iterate over commands with no duplication
    pub fn iter(&self) -> impl Iterator<Item = MRc<dyn Cmd + Sync + Send>> + '_ {
        let mut dups = HashSet::new();
        let mut cmds = Vec::new();
        for cmd in self.search.values() {
            let names = cmd.lock().commands();
            if dups.contains(names) {
                continue;
            }
            dups.insert(names);
            cmds.push(cmd.clone());
        }
        cmds.into_iter()
    }
}

#[cfg(test)]
mod commands_test {
    use super::Commands;
    use crate::utils::Quit;
    use alloc::sync::Arc;
    use parking_lot::Mutex;

    #[test]
    fn test_iter() {
        let mut cmds = Commands::default();
        cmds.add_command("q", Arc::new(Mutex::new(Quit)));
        cmds.add_command("quit", Arc::new(Mutex::new(Quit)));
        assert_eq!(cmds.iter().count(), 1);
    }
}
