mod addresses;
mod ascii;
mod command;
mod editor;
mod events;
mod help;
mod hex;
mod mode;
mod render;
mod selector;

use mode::Mode;
use rair_core::{error_msg, expect, Cmd, Core};
use ratatui::{
    crossterm::event::{self, Event},
    prelude::*,
    Terminal,
};
use std::io;

use crate::visual_hex::VisualHexWithoutEnv;

enum NextAction {
    Continue,
    Break,
}

pub struct VisualHex {
    internal_error: Option<String>,
    mode: Mode,
    rendered_bytes: u64,
}

impl VisualHex {
    pub fn new(core: &mut Core) -> Self {
        // we cannot have it as part of visual hex for mutability issues
        let _ = VisualHexWithoutEnv::new(core);
        Self {
            internal_error: None,
            mode: Mode::default(),
            rendered_bytes: 0,
        }
    }
    fn render_event_loop<B: Backend>(
        &mut self,
        mut term: Terminal<B>,
        core: &mut Core,
    ) -> io::Result<()> {
        let mut notenv = VisualHexWithoutEnv::new(core);
        loop {
            let env = notenv.get_env(core);
            term.draw(|f| self.render(f, core, env))?;
            if let Some(e) = self.internal_error.take() {
                return Err(io::Error::other(e));
            }
            if let Event::Key(key) = event::read()? {
                if let NextAction::Break = self.handle_input(key, core, env) {
                    break;
                };
            }
        }
        Ok(())
    }
}
impl Cmd for VisualHex {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if !args.is_empty() {
            expect(core, args.len() as u64, 1);
            return;
        }
        let terminal = ratatui::init();
        let loc = core.get_loc();
        if let Err(e) = self.render_event_loop(terminal, core) {
            core.set_loc(loc);
            ratatui::restore();
            error_msg(core, "Error in Visual mode event loop", &e.to_string());
            return;
        }
        core.set_loc(loc);
        ratatui::restore();
    }
    fn commands(&self) -> &'static [&'static str] {
        &["visualHex", "vx"]
    }

    fn help_messages(&self) -> &'static [(&'static str, &'static str)] {
        &[("", "Open hex editor in visual mode.")]
    }
}
