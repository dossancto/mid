use std::collections::{BTreeSet, HashMap};

use color_eyre::Result;
use crossterm::event::{self, KeyCode};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Row, Table, TableState};

use crate::core::databases::application::query::DbValue;
use crate::core::query::TableCommand;

#[derive(Debug, Clone)]
pub struct TableEvent {
    pub key_code: KeyCode,
    pub value: Option<String>,
}

pub fn render_output_as_table(
    items: Vec<HashMap<String, DbValue>>,
    table_commands: Vec<TableCommand>,
) -> Result<Option<TableEvent>> {
    let mut table_state = TableState::default();
    let mut column_offset = 0usize;
    let mut selected_column = 0usize;
    table_state.select_first();
    ratatui::run(
        |terminal| -> std::result::Result<Option<TableEvent>, color_eyre::eyre::Error> {
            loop {
                terminal.draw(|frame| {
                    render(
                        frame,
                        &mut table_state,
                        &mut column_offset,
                        &mut selected_column,
                        &items,
                    )
                })?;
                if let Some(key) = event::read()?.as_key_press_event() {
                    if table_commands.contains(&TableCommand::Moviment) {
                        match key.code.clone() {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(None),
                            KeyCode::Char('j') | KeyCode::Down => table_state.select_next(),
                            KeyCode::Char('k') | KeyCode::Up => table_state.select_previous(),
                            KeyCode::Char('l') | KeyCode::Right => {
                                selected_column = selected_column.saturating_add(1)
                            }
                            KeyCode::Char('h') | KeyCode::Left => {
                                selected_column = selected_column.saturating_sub(1)
                            }
                            KeyCode::Char('g') => table_state.select_first(),
                            KeyCode::Char('G') => table_state.select_last(),
                            _ => {}
                        }
                    }

                    if table_commands.contains(&TableCommand::DatabaseTable) {
                        match key.code {
                            KeyCode::Enter
                                if table_commands.contains(&TableCommand::DatabaseTable) =>
                            {
                                let Some(selected_row) = table_state.selected() else {
                                    continue;
                                };
                                let Some(table_name) = items
                                    .get(selected_row)
                                    .and_then(|row| row.get("table_name"))
                                    .map(format_db_value)
                                else {
                                    continue;
                                };

                                return Ok(Some(TableEvent {
                                    key_code: KeyCode::Enter,
                                    value: Some(table_name),
                                }));
                            }
                            _ => {}
                        }
                    }
                }
            }
        },
    )
}

/// Render the UI with a table.
fn render(
    frame: &mut Frame,
    table_state: &mut TableState,
    column_offset: &mut usize,
    selected_column: &mut usize,
    items: &[HashMap<String, DbValue>],
) {
    let layout = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(1);
    let [top, main] = frame.area().layout(&layout);

    let title = Line::from_iter([
        Span::from("Table Widget").bold(),
        Span::from(" (Press 'q' to quit, j/k rows, h/l columns)"),
    ]);
    frame.render_widget(title.centered(), top);

    render_table(
        frame,
        main,
        table_state,
        column_offset,
        selected_column,
        items,
    );
}

/// Render a table with some rows and columns.
pub fn render_table(
    frame: &mut Frame,
    area: Rect,
    table_state: &mut TableState,
    column_offset: &mut usize,
    selected_column: &mut usize,
    items: &[HashMap<String, DbValue>],
) {
    const COLUMN_WIDTH: u16 = 20;
    const COLUMN_SPACING: u16 = 1;

    let (headers, row_values) = format_to_table_elements(items);
    let max_selected_column = headers.len().saturating_sub(1);
    *selected_column = (*selected_column).min(max_selected_column);

    let visible_columns = ((area.width.saturating_add(COLUMN_SPACING))
        / (COLUMN_WIDTH + COLUMN_SPACING))
        .max(1) as usize;

    if *selected_column < *column_offset {
        *column_offset = *selected_column;
    } else if *selected_column >= *column_offset + visible_columns {
        *column_offset = selected_column
            .saturating_add(1)
            .saturating_sub(visible_columns);
    }

    let max_offset = headers.len().saturating_sub(visible_columns);
    *column_offset = (*column_offset).min(max_offset);
    let visible_end = (*column_offset + visible_columns).min(headers.len());
    let selected_visible_column = selected_column.saturating_sub(*column_offset);

    let visible_headers = headers[*column_offset..visible_end].to_vec();

    let header = Row::new(visible_headers.clone())
        .style(Style::new().bold())
        .bottom_margin(1);

    let rows: Vec<Row> = row_values
        .into_iter()
        .map(|row| {
            Row::new(
                row.into_iter()
                    .skip(*column_offset)
                    .take(visible_columns)
                    .collect::<Vec<String>>(),
            )
        })
        .collect();
    let widths: Vec<Constraint> = visible_headers
        .iter()
        .map(|_| Constraint::Length(COLUMN_WIDTH))
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .column_spacing(COLUMN_SPACING)
        .style(Color::White)
        .row_highlight_style(Style::new().on_black().bold())
        .column_highlight_style(Color::Gray)
        .cell_highlight_style(Style::new().reversed().yellow())
        .highlight_symbol("> ");

    table_state.select_column(Some(selected_visible_column));
    frame.render_stateful_widget(table, area, table_state);
}

fn format_to_table_elements(items: &[HashMap<String, DbValue>]) -> (Vec<String>, Vec<Vec<String>>) {
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
                        .map(format_db_value)
                        .unwrap_or_else(|| "null".to_string())
                })
                .collect::<Vec<String>>()
        })
        .collect::<Vec<Vec<String>>>();

    (headers, rows)
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
