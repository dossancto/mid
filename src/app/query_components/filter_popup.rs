use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    text::Line,
    widgets::{Block, Clear, Paragraph, Widget},
};

#[derive(Default)]
pub(crate) struct FilterPopup {
    visible: bool,
    input: String,
}

impl FilterPopup {
    pub(crate) fn open(&mut self) {
        self.visible = true;
        self.input.clear();
    }

    pub(crate) fn reset(&mut self) {
        self.close();
    }

    pub(crate) fn handle_key_event(&mut self, key_event: KeyEvent) -> Option<String> {
        match key_event.code {
            KeyCode::Char(character) => self.input.push(character),
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Enter => {
                let input = self.input.clone();
                self.close();
                return Some(input);
            }
            KeyCode::Esc => self.close(),
            _ => {}
        }

        None
    }

    fn close(&mut self) {
        self.visible = false;
        self.input.clear();
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Widget for &FilterPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.visible {
            return;
        }

        let popup_area = area.centered(Constraint::Length(40), Constraint::Length(3));
        Clear.render(popup_area, buf);

        Paragraph::new(vec![Line::from(format!("{}▏", self.input))])
            .block(Block::bordered().title(" Search "))
            .render(popup_area, buf);
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::FilterPopup;

    #[test]
    fn returns_text_filter_input() {
        let mut popup = FilterPopup::default();
        popup.open();
        for character in ['a', '4', '2'] {
            popup.handle_key_event(KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE));
        }

        assert_eq!(
            popup.handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
            Some("a42".to_string())
        );
    }
}
