use super::{mode::Mode, VisualHex};
use crate::visual_hex::VisualHexEnv;
use rair_core::Core;
use ratatui::prelude::*;
use std::collections::BTreeMap;

impl VisualHex {
    fn hex_line<'a>(&self, loc: u64, data: &BTreeMap<u64, u8>, env: &VisualHexEnv) -> Line<'a> {
        let mut spans = Vec::with_capacity(16);
        for j in 0..16 {
            let hex = if let Some(c) = data.get(&(j + loc)) {
                format!("{:02x}", c)
            } else {
                format!("{}{}", env.non_visual.gap, env.non_visual.gap)
            };
            match &self.mode {
                Mode::Edit(editor) => spans.extend_from_slice(&editor.span_hex(hex, loc + j)),
                Mode::Select(selector) => spans.push(selector.span_hex(hex, loc + j)),
                _ => spans.push(Span::from(hex)),
            }
            if j % 2 == 1 {
                spans.push(Span::from(" "));
            }
        }
        Line::default().spans(spans)
    }
    pub(super) fn render_hex(
        &self,
        f: &mut Frame,
        chunk: Rect,
        core: &Core,
        data: &BTreeMap<u64, u8>,
        env: &VisualHexEnv,
    ) {
        let mut lines: Vec<Line<'_>> = Vec::with_capacity(chunk.height as usize + 1);
        lines.push(env.render_hex_banner());

        let loc = core.get_loc();

        for i in 0..chunk.height as u64 {
            lines.push(self.hex_line(loc + i * 16, data, env));
        }
        let txt = Text::from(lines);
        f.render_widget(txt, chunk);
    }
}
