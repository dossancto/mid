use std::io;

use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Clear, Paragraph, Widget},
};

const FIELD_COUNT: usize = 6;

#[derive(Clone, Copy, Default)]
enum DatabaseKind {
    #[default]
    Postgres,
    MySql,
}

impl DatabaseKind {
    fn scheme(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::MySql => "mysql",
        }
    }

    fn default_port(self) -> u16 {
        match self {
            Self::Postgres => 5432,
            Self::MySql => 3306,
        }
    }

    fn toggle(&mut self) {
        *self = match self {
            Self::Postgres => Self::MySql,
            Self::MySql => Self::Postgres,
        };
    }
}

pub struct RemoteAddScreen {
    host: String,
    port: String,
    username: String,
    password: String,
    database: String,
    database_type: DatabaseKind,
    selected_field: usize,
    error: Option<String>,
    connection_string: Option<String>,
    exit: bool,
}

impl Default for RemoteAddScreen {
    fn default() -> Self {
        let database_type = DatabaseKind::default();
        Self {
            host: "localhost".to_string(),
            port: database_type.default_port().to_string(),
            username: String::new(),
            password: String::new(),
            database: String::new(),
            database_type,
            selected_field: 0,
            error: None,
            connection_string: None,
            exit: false,
        }
    }
}

impl RemoteAddScreen {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<Option<String>> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&*self, frame.area()))?;

            if let Event::Key(key_event) = event::read()?
                && matches!(key_event.kind, KeyEventKind::Press | KeyEventKind::Repeat)
            {
                self.handle_key_event(key_event);
            }
        }

        Ok(self.connection_string.clone())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        self.error = None;

        match key_event.code {
            KeyCode::Char('q') => self.exit = true,
            KeyCode::Char('v')
                if key_event.kind == KeyEventKind::Press
                    && key_event.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.paste_clipboard();
            }
            KeyCode::Tab | KeyCode::Down => {
                self.selected_field = (self.selected_field + 1) % FIELD_COUNT;
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.selected_field = self
                    .selected_field
                    .checked_sub(1)
                    .unwrap_or(FIELD_COUNT - 1);
            }
            KeyCode::Left | KeyCode::Right if self.selected_field == 0 => {
                self.database_type.toggle();
                self.port = self.database_type.default_port().to_string();
            }
            KeyCode::Enter => self.finish(),
            KeyCode::Backspace => {
                if let Some(value) = self.active_value_mut() {
                    value.pop();
                }
            }
            KeyCode::Char(character)
                if !key_event
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
            {
                if self.selected_field != 0
                    && (self.selected_field != 2 || character.is_ascii_digit())
                    && let Some(value) = self.active_value_mut()
                {
                    value.push(character);
                }
            }
            _ => {}
        }
    }

    fn paste_clipboard(&mut self) {
        if self.selected_field == 0 {
            return;
        }

        let text = match Clipboard::new().and_then(|mut clipboard| clipboard.get_text()) {
            Ok(text) => text.trim_end_matches(['\r', '\n']).to_string(),
            Err(error) => {
                self.error = Some(format!("Failed to read clipboard: {error}"));
                return;
            }
        };

        if self.selected_field == 2 && !text.chars().all(|character| character.is_ascii_digit()) {
            self.error = Some("Port must contain only numbers".to_string());
            return;
        }

        if let Some(value) = self.active_value_mut() {
            value.push_str(&text);
        }
    }

    fn active_value_mut(&mut self) -> Option<&mut String> {
        match self.selected_field {
            1 => Some(&mut self.host),
            2 => Some(&mut self.port),
            3 => Some(&mut self.username),
            4 => Some(&mut self.password),
            5 => Some(&mut self.database),
            _ => None,
        }
    }

    fn finish(&mut self) {
        let port = match self.port.parse::<u16>() {
            Ok(port) if port > 0 => port,
            _ => {
                self.error = Some("Port must be between 1 and 65535".to_string());
                return;
            }
        };

        if self.host.trim().is_empty()
            || self.username.trim().is_empty()
            || self.database.trim().is_empty()
        {
            self.error = Some("Host, username, and database are required".to_string());
            return;
        }

        self.connection_string = Some(format!(
            "{}://{}:{}@{}:{}/{}",
            self.database_type.scheme(),
            self.username,
            self.password,
            self.host,
            port,
            self.database,
        ));
        self.exit = true;
    }

    fn field_line<'a>(&self, index: usize, label: &'a str, value: String) -> Line<'a> {
        let marker = if self.selected_field == index {
            "> "
        } else {
            "  "
        };
        let line = Line::from(vec![
            marker.into(),
            format!("{label:<10} ").into(),
            value.into(),
        ]);

        if self.selected_field == index {
            line.style(Style::new().fg(Color::Yellow))
        } else {
            line
        }
    }
}

impl Widget for &RemoteAddScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_area = area.centered(Constraint::Length(64), Constraint::Length(14));
        Clear.render(popup_area, buf);

        let inner_area = popup_area.inner(ratatui::layout::Margin::new(2, 2));
        let [fields_area, status_area, help_area] = Layout::vertical([
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner_area);
        let password = if self.password.is_empty() {
            String::new()
        } else {
            "•".repeat(self.password.chars().count())
        };
        let fields = vec![
            self.field_line(0, "Type", self.database_type.scheme().to_string()),
            self.field_line(1, "Host", self.host.clone()),
            self.field_line(2, "Port", self.port.clone()),
            self.field_line(3, "Username", self.username.clone()),
            self.field_line(4, "Password", password),
            self.field_line(5, "Database", self.database.clone()),
        ];

        Block::bordered()
            .title(" Add remote connection ")
            .render(popup_area, buf);
        Paragraph::new(fields).render(fields_area, buf);

        if let Some(error) = &self.error {
            Paragraph::new(error.as_str())
                .red()
                .render(status_area, buf);
        }
        Paragraph::new("Tab/↑/↓ field  ←/→ type  Enter save  Esc q")
            .dark_gray()
            .render(help_area, buf);
    }
}
