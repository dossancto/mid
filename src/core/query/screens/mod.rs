use std::collections::{BTreeSet, HashMap};

use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span},
    widgets::{Row, Table, TableState},
};

use crate::core::databases::application::query::DbValue;

pub fn format_db_value(value: &DbValue) -> String {
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

fn items_to_rows_elements(items: &[HashMap<String, DbValue>]) -> (Vec<String>, Vec<Vec<String>>) {
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

    let (headers, row_values) = items_to_rows_elements(items);
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
