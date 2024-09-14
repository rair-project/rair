use super::{mode::Mode, VisualHex};
use crate::visual_hex::VisualHexEnv;
use rair_core::Core;
use ratatui::{prelude::*, Frame};

impl VisualHex {
    /// returns total number of bytes to displays
    pub(super) fn calculate_bytes_to_render(&mut self, chunk: &Rect) -> u64 {
        // we multiply the height by 16 because each line will
        // have 16 bytes in total
        let height = chunk.height as u64 - 1;
        self.rendered_bytes = height * 16;
        self.rendered_bytes
    }

    fn render_body(&mut self, f: &mut Frame, chunk: Rect, core: &mut Core, env: &VisualHexEnv) {
        let max_addr = self.max_address_str_width(core, &chunk);
        let chunks = Layout::horizontal([
            Constraint::Length(max_addr), // address
            Constraint::Length(39),       // hex
            Constraint::Length(16),       // ascii
        ])
        .spacing(2)
        .split(chunk);
        self.render_addresses(f, chunks[0], core, env);

        let size = self.calculate_bytes_to_render(&chunk);
        let loc = core.get_loc();

        let data = match core.read_sparce(loc, size) {
            Ok(d) => d,
            Err(e) => {
                self.internal_error = Some(format!("Read Failed, {}", &e.to_string()));
                return;
            }
        };
        self.render_hex(f, chunks[1], core, &data, env);
        self.render_ascii(f, chunks[2], core, &data, env);
    }

    /// render 1 frame, on error this function populates `self.internal_error`I
    pub(super) fn render(&mut self, f: &mut Frame, core: &mut Core, env: &VisualHexEnv) {
        let area = f.area();
        if area.height < 5 || area.width < 70 {
            self.internal_error = Some("Terminal must be at least 5x70".to_owned());
        }
        let mut layout = vec![];
        if env.show_help {
            layout.push(Constraint::Max(2));
        }
        layout.push(Constraint::Min(2)); // body
        if let Mode::Command { .. } = self.mode {
            layout.push(Constraint::Max(1)); // commands
        }
        let chunks = Layout::vertical(layout).split(area);
        let mut idx = 0;
        if env.show_help {
            self.render_help(f, chunks[idx], env);
            idx += 1;
        }
        self.render_body(f, chunks[idx], core, env);
        idx += 1;
        if let Mode::Command(cmd) = &self.mode {
            cmd.render(f, chunks[idx]);
        }
    }
}
