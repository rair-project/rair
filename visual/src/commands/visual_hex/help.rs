use super::{Mode, VisualHex};
use crate::visual_hex::VisualHexEnv;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

impl VisualHex {
    fn view_help<'a>(&self, env: &VisualHexEnv) -> Line<'a> {
        Line::from(vec![
            Span::styled("View", Style::default().bold()),
            Span::from(" mode: Arrow keys nagivate.  Pressing <"),
            Span::styled(env.edit_mode_button.to_string(), Style::default().bold()),
            Span::from("> enters edit mode, <"),
            Span::styled(env.select_mode_button.to_string(), Style::default().bold()),
            Span::from("> enters select mode, <"),
            Span::styled(env.command_mode_button.to_string(), Style::default().bold()),
            Span::from("> enters command mode, <"),
            Span::styled(env.quit_buton.to_string(), Style::default().bold()),
            Span::from("> quits visual mode. Everything else is ignored."),
        ])
    }
    fn command_help<'a>(&self) -> Line<'a> {
        Line::from(vec![
            Span::styled("Command", Style::default().bold()),
            Span::from(" mode: Use the keyboard to type commands.  Pressing <"),
            Span::styled("Enter", Style::default().bold()),
            Span::from("> runs the command and return to view mode, <"),
            Span::styled("ESC", Style::default().bold()),
            Span::from("> returns to view mode without running the command."),
        ])
    }
    fn edit_help<'a>(&self) -> Line<'a> {
        Line::from(vec![
            Span::styled("Edit", Style::default().bold()),
            Span::from(" mode: Arrow keys nagivate.  Pressing <"),
            Span::styled("Tab", Style::default().bold()),
            Span::from("> switches between ascii and hex edit, <"),
            Span::styled("ESC", Style::default().bold()),
            Span::from("> returns to view mode. Use the keyboard to edit"),
        ])
    }
    fn select_help<'a>(&self, is_active: bool, env: &VisualHexEnv) -> Line<'a> {
        let mut help = vec![
            Span::styled("Select", Style::default().bold()),
            Span::from(" mode: Arrow keys nagivate.  Pressing <"),
        ];
        if is_active {
            help.push(Span::styled(
                env.copy_button.to_string(),
                Style::default().bold(),
            ));
            help.push(Span::from("> copies selection to clipboard, <"));
            help.push(Span::styled("ESC", Style::default().bold()));
            help.push(Span::from("> cancels selection, <"));
        } else {
            help.push(Span::styled(
                env.select_button.to_string(),
                Style::default().bold(),
            ));
            help.push(Span::from("> starts selection, <"));
            help.push(Span::styled(
                env.paste_button.to_string(),
                Style::default().bold(),
            ));
            help.push(Span::from("> paste from clipboard, <"));
            help.push(Span::styled("ESC", Style::default().bold()));
            help.push(Span::from("> returns to view mode <"));
        }
        help.push(Span::styled(
            env.quit_buton.to_string(),
            Style::default().bold(),
        ));
        help.push(Span::from(
            "> quits visual mode. Everything else is ignored.",
        ));

        Line::from(help)
    }
    fn help<'a>(&self, env: &VisualHexEnv) -> Line<'a> {
        match &self.mode {
            Mode::View => self.view_help(env),
            Mode::Edit(_) => self.edit_help(),
            Mode::Command(_) => self.command_help(),
            Mode::Select(selector) => self.select_help(selector.is_active(), env),
        }
    }
    /// display help based on mode
    pub(super) fn render_help(&self, f: &mut Frame, chunk: Rect, env: &VisualHexEnv) {
        let p = Paragraph::new(self.help(env)).wrap(Wrap { trim: true });
        f.render_widget(p, chunk);
    }
}
