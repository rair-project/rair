//! Data Structure for holding rair commands.

use std::collections::HashSet;

use crate::{cmds::Cmd, helper::MRc};
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
    use crate::{Cmd, Core};
    use alloc::sync::Arc;
    use parking_lot::Mutex;

    #[derive(Default)]
    pub struct Foo;

    impl Cmd for Foo {
        fn run(&mut self, _core: &mut Core, _args: &[String]) {
            todo!()
        }
        fn commands(&self) -> &'static [&'static str] {
            &["foo", "f"]
        }

        fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
            todo!()
        }
    }

    #[test]
    fn test_iter() {
        let mut cmds = Commands::default();
        cmds.add_command("f", Arc::new(Mutex::new(Foo)));
        cmds.add_command("foo", Arc::new(Mutex::new(Foo)));
        assert_eq!(cmds.iter().count(), 1);
    }
}
