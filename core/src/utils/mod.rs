//! Utility commands.
mod env;
mod project;
mod quit;

use self::env::{Environment, EnvironmentHelp, EnvironmentReset};
use self::project::{Load, Save};
pub use self::quit::Quit;
use crate::core::Core;
use alloc::sync::Arc;
use parking_lot::Mutex;

pub fn register_utils(core: &mut Core) {
    core.add_command("quit", "q", Arc::new(Mutex::new(Quit::new())));
    core.add_command("save", "", Arc::new(Mutex::new(Save::new())));
    core.add_command("load", "", Arc::new(Mutex::new(Load::new())));
    core.add_command("environment", "e", Arc::new(Mutex::new(Environment::new())));
    core.add_command(
        "environmentReset",
        "er",
        Arc::new(Mutex::new(EnvironmentReset::new())),
    );
    let eh = Arc::new(Mutex::new(EnvironmentHelp::new(core)));
    core.add_command("environmentHelp", "eh", eh);
}
