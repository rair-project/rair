//! Linking all rair parts together into 1 module.

use crate::cmd::{Cmd, CmdOps};
use crate::commands::Commands;
use crate::helper::{error_msg, AddrMode};
use crate::io::register_io;
use crate::loc::register_loc;
use crate::utils::register_utils;
use crate::writer::Writer;
use alloc::sync::Arc;
use core::mem;
use parking_lot::{Mutex, RwLock};
use rair_env::Environment;
use rair_io::RIO;
use serde::{Deserialize, Serialize};
use std::io;
use std::io::Write;
use yansi::Paint;
#[derive(Serialize, Deserialize)]
pub struct Core {
    pub mode: AddrMode,
    pub io: RIO,
    loc: u64,
    // Every time you add some new serde(skip) variable
    // make sure that this variable is well initialized
    // in the projects commands.
    #[serde(skip)]
    pub stdout: Writer,
    #[serde(skip)]
    pub stderr: Writer,
    #[serde(skip)]
    commands: Arc<Mutex<Commands>>,
    #[serde(skip)]
    pub env: Arc<RwLock<Environment<Core>>>,
}

impl Default for Core {
    fn default() -> Self {
        Core {
            mode: AddrMode::Phy,
            stdout: Writer::new_write(Box::new(io::stdout())),
            stderr: Writer::new_write(Box::new(io::stderr())),
            io: RIO::new(),
            loc: 0,
            commands: Arc::default(),
            env: Arc::default(),
        }
    }
}
fn set_global_color(_: &str, value: bool, _: &Environment<Core>, _: &mut Core) -> bool {
    if value {
        yansi::enable();
    } else {
        yansi::disable();
    }
    true
}
impl Core {
    pub(crate) fn load_commands(&mut self) {
        register_io(self);
        register_loc(self);
        register_utils(self);
    }
    /// Returns list of all available commands in [Core].
    pub fn commands(&mut self) -> Arc<Mutex<Commands>> {
        self.commands.clone()
    }
    pub fn set_commands(&mut self, commands: Arc<Mutex<Commands>>) {
        self.commands = commands;
    }
    fn init_core_env(&mut self) {
        let locked_env = self.env.clone();
        let mut env = locked_env.write();

        env.add_bool(
            "core.helpInvalidCommand",
            true,
            "Show help for suggestions in case of invalid Command",
        )
        .unwrap();
    }
    fn init_colors(&mut self, enable: bool) {
        let locked_env = self.env.clone();
        let mut env = locked_env.write();
        env.add_bool_with_cb(
            "color.enable",
            enable,
            "Enable/Disable color theme globally",
            self,
            set_global_color,
        )
        .unwrap();
        env.add_color("color.1", (0x58, 0x68, 0x75), "").unwrap();
        env.add_color("color.2", (0xb5, 0x89, 0x00), "").unwrap();
        env.add_color("color.3", (0xcb, 0x4b, 0x16), "").unwrap();
        env.add_color("color.4", (0xdc, 0x32, 0x2f), "").unwrap();
        env.add_color("color.5", (0xd3, 0x36, 0x82), "").unwrap();
        env.add_color("color.6", (0x6c, 0x71, 0xc4), "").unwrap();
        env.add_color("color.7", (0x26, 0x8b, 0xd2), "").unwrap();
        env.add_color("color.8", (0x2a, 0xa1, 0x98), "").unwrap();
        env.add_color("color.9", (0x85, 0x99, 0x00), "").unwrap();
    }
    fn new_settings(color: bool) -> Self {
        let mut core = Core::default();
        core.init_colors(color);
        core.init_core_env();
        core.load_commands();
        core
    }
    #[must_use]
    pub fn new() -> Self {
        Core::new_settings(true)
    }
    #[must_use]
    pub fn new_no_colors() -> Self {
        Core::new_settings(false)
    }
    pub fn set_loc(&mut self, loc: u64) {
        self.loc = loc;
    }

    #[must_use]
    pub fn get_loc(&self) -> u64 {
        self.loc
    }
    pub fn add_command<T: Cmd + Sync + Send + 'static>(&mut self, funcs: T) {
        let cmds = funcs.commands();
        let funcs = Arc::new(Mutex::new(funcs));
        for cmd in cmds {
            if !self.commands.lock().add_command(cmd, funcs.clone()) {
                let msg = format!("Command {} already existed.", cmd.bold().primary());
                error_msg(self, "Cannot add this command.", &msg);
            }
        }
    }
    fn command_not_found(&mut self, command: &str) {
        let msg = format!("Command {} is not found.", command.primary().bold());
        error_msg(self, "Execution failed", &msg);
        let commands = self.commands.lock();
        let similar = commands.suggest(command, 2);
        let (r, g, b) = self.env.read().get_color("color.6").unwrap();
        if !similar.is_empty() {
            write!(self.stderr, "Similar command: {}", similar[0].rgb(r, g, b)).unwrap();
            if similar.len() > 1 {
                for suggestion in &similar[1..] {
                    write!(self.stderr, ", {}", suggestion.rgb(r, g, b)).unwrap();
                }
            }
            writeln!(self.stderr, ".").unwrap();
        }
        let show_help = self.env.read().get_bool("core.helpInvalidCommand").unwrap();
        if show_help {
            let similar: Vec<String> = similar.iter().map(|s| (*s).to_owned()).collect();
            drop(commands);
            for suggestion in similar {
                self.help(&suggestion.clone());
            }
        }
    }

    pub fn run(&mut self, command: &str, args: &[String]) {
        let cmds = self.commands.clone();
        let cmds_ref = cmds.lock();
        let cmd = cmds_ref.find(command);
        if let Some(cmd) = cmd {
            cmd.lock().run(self, args);
        } else {
            drop(cmds_ref);
            self.command_not_found(command);
        }
    }

    pub fn run_at(&mut self, command: &str, args: &[String], at: u64) {
        let old_loc = mem::replace(&mut self.loc, at);
        self.run(command, args);
        self.loc = old_loc;
    }

    pub fn help(&mut self, command: &str) {
        let cmds = self.commands.clone();
        let cmds_ref = cmds.lock();
        let cmd = cmds_ref.find(command);
        if let Some(cmd) = cmd {
            let cmd = cmd.as_ref().lock();
            cmd.help(self);
        } else {
            drop(cmds_ref);
            self.command_not_found(command);
        }
    }
}

#[cfg(test)]
mod test_core {
    use super::*;
    use crate::utils::Quit;
    fn testings_env(core: &mut Core) {
        let locked_env = core.env.clone();
        let mut env = locked_env.write();
        env.set_bool("core.helpInvalidCommand", false, core)
            .unwrap();
    }
    #[test]
    fn test_loc() {
        let mut core = Core::new_no_colors();
        core.set_loc(0x500);
        assert_eq!(core.get_loc(), 0x500);
    }
    #[test]
    fn test_add_command() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.add_command(Quit);
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Cannot add this command.\nCommand quit already existed.\nError: Cannot add this command.\nCommand q already existed.\n"
        );
    }
    #[test]
    fn test_help_failure() {
        let mut core = Core::new_no_colors();
        testings_env(&mut core);
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.help("seeker");
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Execution failed\nCommand seeker is not found.\nSimilar command: seek.\n"
        );
    }
    #[test]
    fn test_run_at() {
        let mut core = Core::new_no_colors();
        testings_env(&mut core);
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run_at("mep", &[], 0x500);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Execution failed\nCommand mep is not found.\nSimilar command: map, maps, m, e, er, eh.\n"
        );
    }
    #[test]
    fn test_help_failure_with_extras() {
        let mut core = Core::new_no_colors();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.help("seeker");
        assert_eq!(core.stdout.utf8_string().unwrap(), "Commands: [seek | s]\nUsage:\ns +\t\tRedo Seek.\ns -\t\tUndo Seek.\ns +[offset]\tIncrease current loc by offset.\ns -[offset]\tDecrease current loc by offset.\ns [offset]\tSet current location to offset.\n");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Execution failed\nCommand seeker is not found.\nSimilar command: seek.\n"
        );
    }
}
