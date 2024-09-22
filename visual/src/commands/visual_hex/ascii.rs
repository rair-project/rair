use crate::visual_hex::VisualHexEnv;

use super::{mode::Mode, VisualHex};
use rair_core::Core;
use ratatui::prelude::*;
use std::collections::BTreeMap;

impl VisualHex {
    fn ascii_line<'a>(
        &self,
        loc: u64,
        data: &BTreeMap<u64, u8>,
        env: &'a VisualHexEnv,
    ) -> Line<'a> {
        let mut spans = Vec::with_capacity(16);
        for j in 0..16 {
            let byte = data.get(&(j + loc)).copied();
            let span = env.render_ascii(byte);
            let span = match &self.mode {
                Mode::Edit(editor) => editor.span_ascii(span, loc + j),
                Mode::Select(selector) => selector.span_ascii(span, loc + j),
                _ => span,
            };
            spans.push(span);
        }
        Line::default().spans(spans)
    }
    pub(super) fn render_ascii(
        &self,
        f: &mut Frame,
        chunk: Rect,
        core: &Core,
        data: &BTreeMap<u64, u8>,
        env: &VisualHexEnv,
    ) {
        let mut lines: Vec<Line<'_>> = Vec::with_capacity(chunk.height as usize + 1);
        lines.push(env.render_ascii_banner());

        let loc = core.get_loc();

        for i in 0..chunk.height as u64 {
            lines.push(self.ascii_line(loc + i * 16, data, env));
        }
        let txt = Text::from(lines);
        f.render_widget(txt, chunk);
    }
}
