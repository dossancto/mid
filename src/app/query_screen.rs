use std::{
    cmp::Ordering,
    io,
    time::{Duration, Instant},
};

use arboard::Clipboard;
use crossterm::{
    event::{
        self, Event, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    terminal::supports_keyboard_enhancement,
};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::Line,
    widgets::{Paragraph, StatefulWidget, TableState, Widget},
};

use super::query_components::{
    FilterPopup, Footer, GotoPopup, ResultsTable, ResultsTableData, format_db_value,
    format_db_value_preview,
};

use super::keybinds_events::KeybindEvents;

use crate::core::{
    config::manage,
    databases::adapters::database_type::{DatabaseHandler, DbValue, QueryResult},
    query::{TableCommand, TableEvent},
};

struct KeyboardEnhancementGuard {
    enabled: bool,
}

impl KeyboardEnhancementGuard {
    fn enable() -> io::Result<Self> {
        let enabled = matches!(supports_keyboard_enhancement(), Ok(true));

        if enabled {
            crossterm::execute!(
                io::stdout(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                )
            )?;
        }

        Ok(Self { enabled })
    }
}

impl Drop for KeyboardEnhancementGuard {
    fn drop(&mut self) {
        if self.enabled {
            let _ = crossterm::execute!(io::stdout(), PopKeyboardEnhancementFlags);
        }
    }
}

#[derive(Clone, Copy)]
enum SortMode {
    Asc,
    Desc,
}

#[derive(Default)]
pub struct QueryScreen {
    items: QueryResult,
    visible_headers: Vec<usize>,
    table_data: ResultsTableData,
    command: TableCommand,
    query: String,
    exit: bool,
    table_state: TableState,
    column_offset: usize,
    selected_column: usize,
    clipboard: Option<Clipboard>,
    event: Option<TableEvent>,
    duration: Duration,
    goto_popup: GotoPopup,
    filter_popup: FilterPopup,
    copied_cell: Option<(usize, usize, Instant)>,
    select_mode: bool,
    select_values: Vec<(usize, usize)>,
    sort_column: Option<usize>,
    sort_mode: Option<SortMode>,
}

impl QueryScreen {
    pub fn new(items: QueryResult, command: TableCommand, query: String) -> Self {
        let mut table_state = TableState::default();
        if !items.rows.is_empty() {
            table_state.select_first();
        }

        let table_data = ResultsTableData::new(&items);
        let visible_headers = (0..items.rows.len()).collect();

        Self {
            items,
            visible_headers,
            table_data,
            command,
            query,
            table_state,
            clipboard: Clipboard::new().ok(),
            ..Self::default()
        }
    }

    pub fn update_app_results(
        &mut self,
        result: crate::core::databases::adapters::database_type::QueryResult,
        query: String,
        duration: Duration,
    ) {
        self.table_data = ResultsTableData::default();
        self.items = result;
        self.visible_headers = (0..self.items.rows.len()).collect();
        self.table_data =
            ResultsTableData::new_filtered(&self.items, &self.visible_headers, &self.items.headers);
        self.query = query;
        self.duration = duration;
        self.exit = false;
        self.event = None;
        self.goto_popup.reset();
        self.filter_popup.reset();
        self.copied_cell = None;
        self.select_mode = false;
        self.select_values.clear();
        self.sort_column = None;
        self.sort_mode = None;
        self.column_offset = 0;
        self.selected_column = 0;
        self.table_state = TableState::default();
        if !self.items.rows.is_empty() {
            self.table_state.select_first();
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<Option<TableEvent>> {
        let _keyboard_enhancement = KeyboardEnhancementGuard::enable()?;

        while !self.exit {
            self.expire_copied_highlight();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(self.event.take())
    }

    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Some((_, _, deadline)) = self.copied_cell {
            let timeout = deadline.saturating_duration_since(Instant::now());
            if !event::poll(timeout)? {
                return Ok(());
            }
        }

        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event)
                if key_event.kind == KeyEventKind::Press
                    || (key_event.kind == KeyEventKind::Repeat
                        && !key_event
                            .modifiers
                            .intersects(KeyModifiers::SHIFT | KeyModifiers::CONTROL)) =>
            {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.goto_popup.is_visible() {
            if let Some(index) = self
                .goto_popup
                .handle_key_event(key_event, self.visible_headers.len())
            {
                self.table_state.select(Some(index));
            }
            return;
        }

        if self.filter_popup.is_visible() {
            if let Some(filter) = self.filter_popup.handle_key_event(key_event) {
                self.apply_filter(&filter);
            }
            return;
        }

        if self.select_mode {
            if let Some(event) = KeybindEvents::parse_to_event(&key_event, &self.command) {
                match event {
                    KeybindEvents::NextRow => self.select_next_row(),
                    KeybindEvents::PreviousRow => self.select_previous_row(),
                    KeybindEvents::NextColumn => self.select_next_column(),
                    KeybindEvents::PreviousColumn => self.select_previous_column(),
                    KeybindEvents::FirstRow => self.select_first_row(),
                    KeybindEvents::LastRow => self.select_last_row(),
                    KeybindEvents::Quit => self.exit(),
                    KeybindEvents::OpenValue => self.select_value(),
                    KeybindEvents::UpdateSelection => self.update_selected_values(),
                    KeybindEvents::SelectMode => self.select_mode(),
                    KeybindEvents::OpenValueInSelectMode => self.open_value_in_select_mode(),
                    _ => {}
                }
            }
            return;
        }

        if let Some(event) = KeybindEvents::parse_to_event(&key_event, &self.command) {
            match event {
                KeybindEvents::NextRow => self.select_next_row(),
                KeybindEvents::PreviousRow => self.select_previous_row(),
                KeybindEvents::NextColumn => self.select_next_column(),
                KeybindEvents::PreviousColumn => self.select_previous_column(),
                KeybindEvents::FirstRow => self.select_first_row(),
                KeybindEvents::LastRow => self.select_last_row(),
                KeybindEvents::YankSelection => self.yank_selected_row(),
                KeybindEvents::UpdateSelection => self.update_selected_value(),
                KeybindEvents::EditQuery => self.edit_query(),
                KeybindEvents::TableSearch => self.select_table(),
                KeybindEvents::GoToRow => self.goto_index(),
                KeybindEvents::Filter => self.filter_popup.open(),
                KeybindEvents::OpenValue => self.open_selected_row(),
                KeybindEvents::Quit => self.exit(),
                KeybindEvents::SelectMode => self.select_mode(),
                KeybindEvents::SortByColumn => self.sort_by_column(),
                _ => {}
            }
        }
    }

    fn sort_by_column(&mut self) {
        let Some(column_index) = self.selected_column_index() else {
            return;
        };

        if self.sort_column != Some(column_index) {
            self.sort_column = Some(column_index);
            self.sort_mode = Some(SortMode::Asc);
        } else {
            self.sort_mode = match self.sort_mode {
                Some(SortMode::Asc) => Some(SortMode::Desc),
                Some(SortMode::Desc) => None,
                None => Some(SortMode::Asc),
            };
        }

        self.visible_headers.sort_unstable();
        match self.sort_mode {
            Some(SortMode::Asc) => self.visible_headers.sort_by(|left, right| {
                compare_db_values(
                    self.items.rows[*left].get(column_index),
                    self.items.rows[*right].get(column_index),
                )
            }),
            Some(SortMode::Desc) => self.visible_headers.sort_by(|left, right| {
                compare_db_values(
                    self.items.rows[*right].get(column_index),
                    self.items.rows[*left].get(column_index),
                )
            }),
            None => {}
        }
        self.table_data =
            ResultsTableData::new_filtered(&self.items, &self.visible_headers, &self.items.headers);
        self.select_values.clear();
        self.copied_cell = None;
    }

    fn open_value_in_select_mode(&mut self) {
        let Some(text) = self.selected_values_text() else {
            return;
        };
        self.event = Some(TableEvent::OpenSelectedRow(text));
        self.exit();
    }

    fn select_mode(&mut self) {
        self.select_mode = !self.select_mode;

        if !self.select_mode {
            self.select_values.clear();
        }
    }

    fn select_value(&mut self) {
        if self.select_mode {
            if let Some(index) = self.selected_item_index() {
                let selected_value = (index, self.selected_column);
                if let Some(position) = self
                    .select_values
                    .iter()
                    .position(|value| *value == selected_value)
                {
                    self.select_values.remove(position);
                } else {
                    self.select_values.push(selected_value);
                }
            }
        }
    }

    fn goto_index(&mut self) {
        self.goto_popup.open();
    }

    fn apply_filter(&mut self, filter: &str) {
        let Some(column_index) = self.selected_column_index() else {
            return;
        };
        let filter = filter.to_lowercase();

        self.visible_headers = (0..self.items.rows.len()).collect();
        self.visible_headers.retain(|index| {
            let value = self.items.rows[*index]
                .get(column_index)
                .map_or_else(|| "null".to_string(), format_db_value);
            filter.is_empty() || value.to_lowercase().contains(&filter)
        });
        self.sort_column = None;
        self.sort_mode = None;
        self.select_values.clear();
        self.table_data =
            ResultsTableData::new_filtered(&self.items, &self.visible_headers, &self.items.headers);
        self.table_state = TableState::default();
        if !self.visible_headers.is_empty() {
            self.table_state.select_first();
        }
        self.copied_cell = None;
    }

    fn selected_item_index(&self) -> Option<usize> {
        self.table_state
            .selected()
            .and_then(|selected| self.visible_headers.get(selected).copied())
    }

    fn edit_query(&mut self) {
        self.event = Some(TableEvent::EditQuery(self.query.clone()));
        self.exit();
    }

    fn update_selected_value(&mut self) {
        let Some(selected_row) = self
            .selected_item_index()
            .and_then(|selected| self.items.rows.get(selected))
        else {
            return;
        };
        let Some(column_index) = self.selected_column_index() else {
            return;
        };
        let Some(value) = selected_row.get(column_index) else {
            return;
        };
        let Some((id_column, id)) = self.row_id(selected_row) else {
            return;
        };
        let Some(table) = Self::table_from_query(&self.query) else {
            return;
        };

        let config = manage::read_config_all().unwrap();
        let database = config.get_database_type().unwrap();
        let update_query = database.update(
            &table,
            id_column,
            id,
            &[(self.items.headers[column_index].as_str(), value)],
        );
        self.event = Some(TableEvent::UpdateValue(format!(
            "-- Save and close to apply this update.\n\
            -- Delete all file to cancel.\n\n\
            {}",
            update_query
        )));
        self.exit();
    }

    fn update_selected_values(&mut self) {
        let Some(table) = Self::table_from_query(&self.query) else {
            return;
        };
        if self.select_values.is_empty() {
            return;
        }

        let headers = self.table_data.headers.iter().cloned().collect::<Vec<_>>();
        let mut selected_by_row = std::collections::BTreeMap::<usize, Vec<usize>>::new();
        for &(row, column) in &self.select_values {
            selected_by_row.entry(row).or_default().push(column);
        }

        let config = manage::read_config_all().unwrap();
        let database = config.get_database_type().unwrap();

        let mut queries = Vec::new();
        for (row_index, mut columns) in selected_by_row {
            let Some(row) = self.items.rows.get(row_index) else {
                continue;
            };
            let Some((id_column, id)) = self.row_id(row) else {
                continue;
            };
            columns.sort_unstable();
            columns.dedup();
            let values = columns
                .iter()
                .filter_map(|&index| Some((headers.get(index)?.as_str(), row.get(index)?)))
                .collect::<Vec<_>>();
            if !values.is_empty() {
                queries.push(database.update(&table, id_column, id, &values));
            }
        }
        if queries.is_empty() {
            return;
        }
        self.event = Some(TableEvent::UpdateValue(format!(
            "-- Save and close to apply these updates.\n-- Delete all file to cancel.\n\n{}",
            queries.join("\n\n")
        )));
        self.exit();
    }

    fn table_from_query(query: &str) -> Option<String> {
        let words = query.split_whitespace().collect::<Vec<_>>();
        let from = words
            .iter()
            .position(|word| word.eq_ignore_ascii_case("FROM"))?;
        words.get(from + 1).map(|table| {
            let table = table.trim_end_matches(|character| character == ';' || character == ',');
            if let Some(table) = table
                .strip_prefix('"')
                .and_then(|table| table.strip_suffix('"'))
            {
                table.replace("\"\"", "\"")
            } else if let Some(table) = table
                .strip_prefix('`')
                .and_then(|table| table.strip_suffix('`'))
            {
                table.replace("``", "`")
            } else {
                table.to_string()
            }
        })
    }

    fn row_id<'a>(&'a self, row: &'a [DbValue]) -> Option<(&'a str, &'a DbValue)> {
        ["Id", "id"].into_iter().find_map(|name| {
            let index = self
                .items
                .headers
                .iter()
                .position(|header| header == name)?;
            Some((self.items.headers[index].as_str(), row.get(index)?))
        })
    }

    fn selected_column_index(&self) -> Option<usize> {
        (self.selected_column < self.items.headers.len()).then_some(self.selected_column)
    }

    fn select_table(&mut self) {
        let Some(table_name) = self
            .selected_item_index()
            .and_then(|selected| self.items.rows.get(selected))
            .and_then(|row| {
                let index = self
                    .items
                    .headers
                    .iter()
                    .position(|header| header == "table_name")?;
                row.get(index)
            })
            .map(format_db_value)
        else {
            return;
        };

        self.event = Some(TableEvent::SelectTable(table_name));
        self.exit();
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_next_row(&mut self) {
        if self.visible_headers.is_empty() {
            return;
        }

        let current_row = self.table_state.selected().unwrap_or(0);
        let last_row = self.visible_headers.len().saturating_sub(1);
        self.table_state
            .select(Some(current_row.saturating_add(1).min(last_row)));
    }

    fn select_previous_row(&mut self) {
        if self.visible_headers.is_empty() {
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

    fn open_selected_row(&mut self) {
        let Some(text) = self.selected_value() else {
            return;
        };
        self.event = Some(TableEvent::OpenSelectedRow(text));
        self.exit();
    }

    fn select_first_row(&mut self) {
        if !self.visible_headers.is_empty() {
            self.table_state.select_first();
        }
    }

    fn select_last_row(&mut self) {
        if !self.visible_headers.is_empty() {
            self.table_state.select_last();
        }
    }

    fn yank_selected_row(&mut self) {
        let Some(text) = self.selected_value() else {
            return;
        };

        let copied = match &mut self.clipboard {
            Some(clipboard) => match clipboard.set_text(text) {
                Ok(()) => true,
                Err(_) => false,
            },
            None => false,
        };

        if copied {
            if let Some(row) = self.table_state.selected() {
                self.copied_cell = Some((
                    row,
                    self.selected_column,
                    Instant::now() + Duration::from_millis(500),
                ));
            }
        }
    }

    fn expire_copied_highlight(&mut self) {
        if self
            .copied_cell
            .is_some_and(|(_, _, deadline)| Instant::now() >= deadline)
        {
            self.copied_cell = None;
        }
    }

    fn copied_cell(&self) -> Option<(usize, usize)> {
        self.copied_cell.map(|(row, column, _)| (row, column))
    }

    fn selected_value(&self) -> Option<String> {
        let Some(row) = self
            .selected_item_index()
            .and_then(|selected| self.items.rows.get(selected))
        else {
            return None;
        };

        let Some(header_index) = self.selected_column_index() else {
            return None;
        };

        Some(
            row.get(header_index)
                .map(|value| match value {
                    DbValue::Json(json) => serde_json::from_str::<serde_json::Value>(json)
                        .and_then(|value| serde_json::to_string_pretty(&value))
                        .unwrap_or_else(|_| json.clone()),
                    value => format_db_value(value),
                })
                .unwrap_or_else(|| "null".to_string()),
        )
    }

    fn selected_value_preview(&self, max_characters: usize) -> Option<String> {
        let row = self
            .selected_item_index()
            .and_then(|selected| self.items.rows.get(selected))?;
        let header_index = self.selected_column_index()?;

        Some(
            row.get(header_index)
                .map(|value| format_db_value_preview(value, max_characters))
                .unwrap_or_else(|| "null".to_string()),
        )
    }

    fn selected_values_text(&self) -> Option<String> {
        if self.select_values.is_empty() {
            return None;
        }

        let mut selected_values = self.select_values.clone();
        selected_values.sort_unstable();

        let mut rows = Vec::new();
        let mut current_row = None;
        let mut row_values = Vec::new();

        for (row_index, column_index) in selected_values {
            if current_row.is_some_and(|current| current != row_index) {
                rows.push(row_values.join(" | "));
                row_values.clear();
            }
            current_row = Some(row_index);

            let value = self
                .items
                .rows
                .get(row_index)
                .and_then(|row| row.get(column_index))
                .map(format_db_value)
                .unwrap_or_else(|| "null".to_string());
            row_values.push(value);
        }

        if !row_values.is_empty() {
            rows.push(row_values.join(" | "));
        }

        Some(rows.join("\n"))
    }

    fn column_count(&self) -> usize {
        self.table_data.headers.len().max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> QueryScreen {
        let mut screen = QueryScreen::default();
        screen.update_app_results(
            QueryResult {
                headers: vec!["name".into(), "id".into()],
                rows: vec![
                    vec![DbValue::Text("Grace".into()), DbValue::Integer(3)],
                    vec![DbValue::Text("Ada".into()), DbValue::Integer(1)],
                    vec![DbValue::Text("Alan".into()), DbValue::Integer(2)],
                ],
            },
            "SELECT name, id FROM users".into(),
            Duration::ZERO,
        );
        screen
    }

    #[test]
    fn sorts_rows_by_column_and_restores_query_order() {
        let mut screen = screen();
        assert_eq!(screen.visible_headers, [0, 1, 2]);
        screen.selected_column = 1;
        screen.sort_by_column();
        assert_eq!(screen.visible_headers, [1, 2, 0]);
        assert_eq!(screen.selected_value().as_deref(), Some("1"));
        screen.sort_by_column();
        assert_eq!(screen.visible_headers, [0, 2, 1]);
        screen.sort_by_column();
        assert_eq!(screen.visible_headers, [0, 1, 2]);
        assert_eq!(screen.table_data.headers, ["name", "id"]);
    }

    #[test]
    fn filters_each_row_and_preserves_filter_when_sorting() {
        let mut screen = screen();
        screen.apply_filter("AL");
        assert_eq!(screen.visible_headers, [2]);
        assert_eq!(screen.selected_value().as_deref(), Some("Alan"));
        screen.sort_by_column();
        assert_eq!(screen.visible_headers, [2]);
        let (header, value) = screen.row_id(&screen.items.rows[2]).unwrap();
        assert_eq!(header, "id");
        assert!(matches!(value, DbValue::Integer(2)));
        screen.apply_filter("missing");
        assert!(screen.selected_value().is_none());
    }
}

fn compare_db_values(left: Option<&DbValue>, right: Option<&DbValue>) -> Ordering {
    match (left, right) {
        (None | Some(DbValue::Null), None | Some(DbValue::Null)) => Ordering::Equal,
        (None | Some(DbValue::Null), _) => Ordering::Greater,
        (_, None | Some(DbValue::Null)) => Ordering::Less,
        (Some(DbValue::Integer(left)), Some(DbValue::Integer(right))) => left.cmp(right),
        (Some(DbValue::Float(left)), Some(DbValue::Float(right))) => {
            left.partial_cmp(right).unwrap_or(Ordering::Equal)
        }
        (Some(DbValue::Boolean(left)), Some(DbValue::Boolean(right))) => left.cmp(right),
        (Some(DbValue::DateTime(left)), Some(DbValue::DateTime(right))) => left.cmp(right),
        (Some(DbValue::Text(left)), Some(DbValue::Text(right))) => left.cmp(right),
        (Some(DbValue::TextArray(left)), Some(DbValue::TextArray(right))) => left.cmp(right),
        (Some(DbValue::Numeric(left)), Some(DbValue::Numeric(right))) => {
            match (left.parse::<f64>(), right.parse::<f64>()) {
                (Ok(left), Ok(right)) => left.partial_cmp(&right).unwrap_or(Ordering::Equal),
                _ => left.cmp(right),
            }
        }
        (Some(DbValue::Json(left)), Some(DbValue::Json(right))) => left.cmp(right),
        (Some(left), Some(right)) => db_value_rank(left).cmp(&db_value_rank(right)),
    }
}

fn db_value_rank(value: &DbValue) -> u8 {
    match value {
        DbValue::Null => 0,
        DbValue::Boolean(_) => 1,
        DbValue::Integer(_) | DbValue::Float(_) | DbValue::Numeric(_) => 2,
        DbValue::Text(_) => 3,
        DbValue::TextArray(_) => 4,
        DbValue::Json(_) => 5,
        DbValue::DateTime(_) => 6,
    }
}

impl Widget for &mut QueryScreen {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let copied_cell = self.copied_cell();

        let [header_area, table_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(Footer::HEIGHT),
        ])
        .areas(area);

        Paragraph::new(Line::from(vec![
            "  Value: ".dark_gray(),
            self.selected_value_preview(200)
                .unwrap_or_default()
                .yellow(),
        ]))
        .render(header_area, buf);

        StatefulWidget::render(
            ResultsTable::new(
                &self.table_data,
                self.selected_column,
                &mut self.column_offset,
                copied_cell,
                &self.select_values,
                self.sort_column
                    .and_then(|index| self.items.headers.get(index).map(String::as_str))
                    .zip(self.sort_mode.map(|mode| matches!(mode, SortMode::Asc))),
            ),
            table_area,
            buf,
            &mut self.table_state,
        );
        Footer::new(
            &self.command,
            self.duration,
            self.items.rows.len(),
            self.table_data.item_count(),
            self.select_mode,
            self.select_values.len(),
        )
        .render(footer_area, buf);

        (&self.goto_popup).render(area, buf);
        (&self.filter_popup).render(area, buf);
    }
}
