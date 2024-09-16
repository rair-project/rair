//! Utility commands.
mod env;
mod project;
mod quit;

use self::env::{Environment, EnvironmentHelp, EnvironmentReset};
use self::project::{Load, Save};
pub use self::quit::Quit;
use crate::core::Core;

pub fn register_utils(core: &mut Core) {
    core.add_command(Quit);
    core.add_command(Save);
    core.add_command(Load);
    core.add_command(Environment);
    core.add_command(EnvironmentReset);
    let eh = EnvironmentHelp::new(core);
    core.add_command(eh);
}
