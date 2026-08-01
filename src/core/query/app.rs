use std::{
    collections::{BTreeSet, HashMap},
    io,
};

use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Paragraph, Row, StatefulWidget, Table, TableState, Widget, Wrap},
};

use crate::core::{
    databases::application::query::DbValue,
    query::{TableCommand, TableEvent},
};

#[derive(Default)]
pub struct App {
    items: Vec<HashMap<String, DbValue>>,
    command: TableCommand,
    query: String,
    query_expanded: bool,
    value_expanded: bool,
    exit: bool,
    table_state: TableState,
    column_offset: usize,
    selected_column: usize,
    clipboard: Option<Clipboard>,
    event: Option<TableEvent>,
}

impl App {
    pub fn new(items: Vec<HashMap<String, DbValue>>, command: TableCommand, query: String) -> Self {
        let mut table_state = TableState::default();
        if !items.is_empty() {
            table_state.select_first();
        }

        Self {
            items,
            command,
            query,
            table_state,
            clipboard: Clipboard::new().ok(),
            ..Self::default()
        }
    }

    fn format_db_value(value: &DbValue) -> String {
        match value {
            DbValue::Null => "null".to_string(),
            DbValue::Text(value) => value.clone(),
            DbValue::TextArray(values) => {
                format!("{{{}}}", values.join(","))
            }
            DbValue::Numeric(value) => value.clone(),
            DbValue::Integer(value) => value.to_string(),
            DbValue::Float(value) => {
                if value.is_finite() {
                    value.to_string()
                } else {
                    "null".to_string()
                }
            }
            DbValue::Boolean(value) => value.to_string(),
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<Option<TableEvent>> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(self.event.take())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if key_event.code == KeyCode::Char('q') {
            self.exit();
            return;
        }

        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.select_next_row(),
            KeyCode::Up | KeyCode::Char('k') => self.select_previous_row(),
            KeyCode::Right | KeyCode::Char('l') => self.select_next_column(),
            KeyCode::Left | KeyCode::Char('h') => self.select_previous_column(),
            KeyCode::Char('g') => self.select_first_row(),
            KeyCode::Char('G') => self.select_last_row(),
            KeyCode::Char('y') => self.yank_selected_row(),
            KeyCode::Char('e') => self.toggle_query_expanded(),
            KeyCode::Char('p') => self.edit_query(),
            KeyCode::Enter => match self.command {
                TableCommand::ShowTables => self.select_table(),
                TableCommand::ShowValue => self.toggle_value_expanded(),
            },
            _ => {}
        }
    }

    fn edit_query(&mut self) {
        self.event = Some(TableEvent::EditQuery(self.query.clone()));
        self.exit();
    }

    fn select_table(&mut self) {
        let Some(table_name) = self
            .table_state
            .selected()
            .and_then(|selected| self.items.get(selected))
            .and_then(|row| row.get("table_name"))
            .map(Self::format_db_value)
        else {
            return;
        };

        self.event = Some(TableEvent::SelectTable(table_name));
        self.exit();
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn toggle_query_expanded(&mut self) {
        self.query_expanded = !self.query_expanded;
    }

    fn toggle_value_expanded(&mut self) {
        if self.selected_value().is_some() {
            self.value_expanded = !self.value_expanded;
        }
    }

    fn select_next_row(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let current_row = self.table_state.selected().unwrap_or(0);
        let last_row = self.items.len().saturating_sub(1);
        self.table_state
            .select(Some(current_row.saturating_add(1).min(last_row)));
    }

    fn select_previous_row(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let current_row = self.table_state.selected().unwrap_or(0);
        self.table_state.select(Some(current_row.saturating_sub(1)));
    }

    fn select_next_column(&mut self) {
        let last_column = self.column_count().saturating_sub(1);
        self.selected_column = self.selected_column.saturating_add(1).min(last_column);
    }

    fn select_previous_column(&mut self) {
        self.selected_column = self.selected_column.saturating_sub(1);
    }

    fn select_first_row(&mut self) {
        if !self.items.is_empty() {
            self.table_state.select_first();
        }
    }

    fn select_last_row(&mut self) {
        if !self.items.is_empty() {
            self.table_state.select_last();
        }
    }

    fn yank_selected_row(&mut self) {
        let Some(text) = self.selected_value() else {
            return;
        };

        let message = match &mut self.clipboard {
            Some(clipboard) => match clipboard.set_text(text) {
                Ok(()) => "Copied selected value".to_string(),
                Err(error) => format!("Clipboard error: {error}"),
            },
            None => "Clipboard is unavailable".to_string(),
        };
        print!("{message}");
    }

    fn selected_value(&self) -> Option<String> {
        let Some(row) = self
            .table_state
            .selected()
            .and_then(|selected| self.items.get(selected))
        else {
            return None;
        };

        let Some(header) = self
            .items
            .iter()
            .flat_map(|row| row.keys().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .nth(self.selected_column)
        else {
            return None;
        };

        Some(
            row.get(&header)
                .map(Self::format_db_value)
                .unwrap_or_else(|| "null".to_string()),
        )
    }

    fn column_count(&self) -> usize {
        self.items
            .iter()
            .flat_map(|row| row.keys())
            .collect::<BTreeSet<_>>()
            .len()
            .max(1)
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        fn items_to_rows_elements(
            items: &[HashMap<String, DbValue>],
        ) -> (Vec<String>, Vec<Vec<String>>) {
            let headers: Vec<String> = items
                .iter()
                .flat_map(|row| row.keys().cloned())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();

            if headers.is_empty() {
                return (
                    vec!["result".to_string()],
                    vec![vec!["No rows".to_string()]],
                );
            }

            let rows = items
                .iter()
                .map(|row| {
                    headers
                        .iter()
                        .map(|header| {
                            row.get(header)
                                .map(App::format_db_value)
                                .unwrap_or_else(|| "null".to_string())
                        })
                        .collect::<Vec<String>>()
                })
                .collect::<Vec<Vec<String>>>();

            (headers, rows)
        }

        // Render the UI with a table.
        let subtitle_lines = if self.query_expanded {
            self.query
                .lines()
                .enumerate()
                .map(|(index, line)| {
                    Line::from(vec![
                        if index == 0 {
                            "Query: ".into()
                        } else {
                            "       ".into()
                        },
                        line.yellow(),
                    ])
                })
                .collect::<Vec<_>>()
        } else {
            vec![Line::from(vec![
                "Query [e to expand]: ".into(),
                self.query.replace(['\n', '\r'], " ").yellow(),
            ])]
        };

        let available_width = usize::from(area.width.max(1));
        let subtitle_height = if self.query_expanded {
            subtitle_lines
                .iter()
                .map(|line| line.width().div_ceil(available_width).max(1))
                .sum::<usize>()
                .min(u16::MAX as usize) as u16
        } else {
            1
        };

        let expanded_value = self.value_expanded.then(|| self.selected_value()).flatten();
        let value_line = expanded_value
            .as_deref()
            .map(|value| Line::from(vec!["Value [Enter to collapse]: ".into(), value.cyan()]));
        let value_height = value_line
            .as_ref()
            .map(|line| line.width().div_ceil(available_width).max(1))
            .unwrap_or(0)
            .min(u16::MAX as usize) as u16;

        let [subtitle_area, value_area, table_area] = Layout::vertical([
            Constraint::Length(subtitle_height),
            Constraint::Length(value_height),
            Constraint::Fill(1),
        ])
        .areas(area);

        let query = Paragraph::new(subtitle_lines);
        if self.query_expanded {
            query.wrap(Wrap { trim: false }).render(subtitle_area, buf);
        } else {
            query.render(subtitle_area, buf);
        }
        if let Some(value_line) = value_line {
            Paragraph::new(value_line)
                .wrap(Wrap { trim: false })
                .render(value_area, buf);
        }

        const COLUMN_WIDTH: u16 = 20;
        const COLUMN_SPACING: u16 = 1;

        let (headers, row_values) = items_to_rows_elements(&self.items);
        self.selected_column = self.selected_column.min(headers.len().saturating_sub(1));

        let visible_columns = ((table_area.width.saturating_add(COLUMN_SPACING))
            / (COLUMN_WIDTH + COLUMN_SPACING))
            .max(1) as usize;

        if self.selected_column < self.column_offset {
            self.column_offset = self.selected_column;
        } else if self.selected_column >= self.column_offset + visible_columns {
            self.column_offset = self
                .selected_column
                .saturating_add(1)
                .saturating_sub(visible_columns);
        }

        self.column_offset = self
            .column_offset
            .min(headers.len().saturating_sub(visible_columns));
        let visible_end = (self.column_offset + visible_columns).min(headers.len());
        let visible_headers = headers[self.column_offset..visible_end].to_vec();

        let header = Row::new(visible_headers.clone())
            .style(Style::new().bold())
            .bottom_margin(1);
        let rows = row_values.into_iter().map(|row| {
            Row::new(
                row.into_iter()
                    .skip(self.column_offset)
                    .take(visible_columns)
                    .collect::<Vec<_>>(),
            )
        });
        let widths = visible_headers
            .iter()
            .map(|_| Constraint::Length(COLUMN_WIDTH))
            .collect::<Vec<_>>();
        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(COLUMN_SPACING)
            // .row_highlight_style(Style::new().reversed())
            .column_highlight_style(Color::DarkGray)
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol("> ");

        self.table_state.select_column(Some(
            self.selected_column.saturating_sub(self.column_offset),
        ));
        StatefulWidget::render(table, table_area, buf, &mut self.table_state);
    }
}
