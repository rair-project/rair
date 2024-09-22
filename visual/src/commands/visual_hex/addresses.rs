use crate::visual_hex::VisualHexEnv;

use super::VisualHex;
use rair_core::Core;
use ratatui::prelude::*;
use std::iter::once;

impl VisualHex {
    /// Finds the maximum width of any address that will be currently rendered
    pub(super) fn max_address_str_width(&mut self, core: &mut Core, chunk: &Rect) -> u16 {
        let loc = core.get_loc();
        let size = self.calculate_bytes_to_render(chunk);
        // we don't put the address of the last bytes, but we put the address
        // and then the last byte is 16 + that address.
        let max_loc = loc.saturating_add(size).saturating_sub(16);
        format!("0x{:08x} ", max_loc).len() as u16
    }
    /// renders the column of all addresses + header
    pub(super) fn render_addresses(
        &mut self,
        f: &mut Frame,
        chunk: Rect,
        core: &mut Core,
        env: &VisualHexEnv,
    ) {
        // TODO center the header
        let banner = env.render_addresses_banner(core);
        let loc = core.get_loc();
        // TODO overflow here
        let addresses = (1..chunk.height as u64).map(|i| env.render_addresses(loc + 16 * (i - 1)));
        let lines: Vec<_> = once(banner).chain(addresses).collect();
        let text = Text::from(lines);
        f.render_widget(text, chunk);
    }
}
