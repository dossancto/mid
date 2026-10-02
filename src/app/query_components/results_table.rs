use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Row, StatefulWidget, Table, TableState},
};

use crate::core::databases::adapters::database_type::QueryResult;

use super::format_db_value_preview;

const COLUMN_WIDTH: u16 = 20;
const COLUMN_SPACING: u16 = 1;
const MAX_CELL_CHARACTERS: usize = 50;

#[derive(Default)]
pub(crate) struct ResultsTableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub source_indices: Vec<usize>,
    pub item_count: usize,
}

impl ResultsTableData {
    pub fn item_count(&self) -> usize {
        self.item_count
    }

    pub(crate) fn new(items: &QueryResult) -> Self {
        let indices = (0..items.rows.len()).collect::<Vec<_>>();
        Self::new_filtered(items, &indices, &[])
    }

    pub(crate) fn new_filtered(
        items: &QueryResult,
        indices: &[usize],
        ordered_headers: &[String],
    ) -> Self {
        let headers = if ordered_headers.is_empty() {
            items.headers.clone()
        } else {
            ordered_headers.to_vec()
        };

        if headers.is_empty() {
            return Self {
                headers: vec!["result".into()],
                rows: vec![vec!["No rows".into()]],
                source_indices: Vec::new(),
                item_count: 0,
            };
        }

        if indices.is_empty() {
            return Self {
                headers: headers,
                rows: vec![vec!["No rows".into()]],
                source_indices: Vec::new(),
                item_count: 0,
            };
        }

        let rows = indices
            .iter()
            .map(|index| {
                items.rows[*index]
                    .iter()
                    .map(|value| format_db_value_preview(value, MAX_CELL_CHARACTERS))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        Self {
            headers,
            rows: rows,
            source_indices: indices.to_vec(),
            item_count: indices.len(),
        }
    }
}

pub(crate) struct ResultsTable<'a> {
    data: &'a ResultsTableData,
    selected_column: usize,
    column_offset: &'a mut usize,
    copied_cell: Option<(usize, usize)>,
    selected_cells: &'a [(usize, usize)],
    sort: Option<(&'a str, bool)>,
}

impl<'a> ResultsTable<'a> {
    pub(crate) fn new(
        data: &'a ResultsTableData,
        selected_column: usize,
        column_offset: &'a mut usize,
        copied_cell: Option<(usize, usize)>,
        selected_cells: &'a [(usize, usize)],
        sort: Option<(&'a str, bool)>,
    ) -> Self {
        Self {
            data,
            selected_column,
            column_offset,
            copied_cell,
            selected_cells,
            sort,
        }
    }
}

impl StatefulWidget for ResultsTable<'_> {
    type State = TableState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let headers = &self.data.headers;
        let row_number_width = self.data.item_count.max(1).to_string().len() as u16;
        let data_width = area.width.saturating_sub(row_number_width + COLUMN_SPACING);
        let visible_columns = ((data_width.saturating_add(COLUMN_SPACING))
            / (COLUMN_WIDTH + COLUMN_SPACING))
            .max(1) as usize;

        match self.selected_column {
            n if n < *self.column_offset => *self.column_offset = n,
            n if n >= *self.column_offset + visible_columns => {
                *self.column_offset = n.saturating_add(1).saturating_sub(visible_columns);
            }
            _ => {}
        }
        *self.column_offset =
            (*self.column_offset).min(headers.len().saturating_sub(visible_columns));

        let visible_end = (*self.column_offset + visible_columns).min(headers.len());
        let visible_headers = headers[*self.column_offset..visible_end].to_vec();
        let mut table_headers = Vec::with_capacity(visible_headers.len() + 1);
        table_headers.push("#".to_string());
        table_headers.extend(visible_headers.iter().map(|header| match self.sort {
            Some((sort_column, true)) if sort_column == header => format!("{header} ↑"),
            Some((sort_column, false)) if sort_column == header => format!("{header} ↓"),
            _ => header.clone(),
        }));
        let header = Row::new(table_headers)
            .style(Style::new().bold())
            .bottom_margin(1);

        let has_items = self.data.item_count > 0;
        let rows = self.data.rows.iter().enumerate().map(|(index, row)| {
            let mut cells = Vec::with_capacity(visible_columns + 1);
            cells.push(if has_items {
                (index + 1).to_string()
            } else {
                String::new()
            });
            cells.extend(
                row.iter()
                    .skip(*self.column_offset)
                    .take(visible_columns)
                    .cloned(),
            );
            Row::new(cells)
        });

        let mut widths = Vec::with_capacity(visible_headers.len() + 1);
        widths.push(Constraint::Length(row_number_width));
        widths.extend(
            visible_headers
                .iter()
                .map(|_| Constraint::Length(COLUMN_WIDTH)),
        );

        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(COLUMN_SPACING)
            .column_highlight_style(Color::DarkGray)
            .cell_highlight_style(Style::new().reversed().yellow())
            .highlight_symbol("> ");

        state.select_column(Some(
            self.selected_column
                .saturating_sub(*self.column_offset)
                .saturating_add(1),
        ));
        StatefulWidget::render(table, area, buf, state);

        let row_offset = state.offset();
        let active_row = state.selected();
        for &(source_row, selected_column) in self.selected_cells {
            let Some(selected_row) = self
                .data
                .source_indices
                .iter()
                .position(|index| *index == source_row)
            else {
                continue;
            };
            if active_row == Some(selected_row) && selected_column == self.selected_column {
                continue;
            }

            let visible_row = selected_row.saturating_sub(row_offset);
            let row_is_visible = selected_row >= row_offset
                && visible_row < usize::from(area.height.saturating_sub(2));
            let column_is_visible = selected_column >= *self.column_offset
                && selected_column < *self.column_offset + visible_columns;

            if row_is_visible && column_is_visible {
                let visible_column = selected_column.saturating_sub(*self.column_offset);
                let x = area
                    .x
                    .saturating_add(2)
                    .saturating_add(row_number_width)
                    .saturating_add(COLUMN_SPACING)
                    .saturating_add(
                        (visible_column as u16).saturating_mul(COLUMN_WIDTH + COLUMN_SPACING),
                    );
                let y = area.y.saturating_add(2).saturating_add(visible_row as u16);
                let selected_area =
                    Rect::new(x, y, COLUMN_WIDTH.min(area.right().saturating_sub(x)), 1);
                buf.set_style(selected_area, Style::new().bg(Color::Blue).fg(Color::White));
            }
        }

        if let Some((copied_row, copied_column)) = self.copied_cell {
            let visible_row = copied_row.saturating_sub(row_offset);
            let row_is_visible = copied_row >= row_offset
                && visible_row < usize::from(area.height.saturating_sub(2));
            let column_is_visible = copied_column >= *self.column_offset
                && copied_column < *self.column_offset + visible_columns;

            if row_is_visible && column_is_visible {
                let visible_column = copied_column - *self.column_offset;
                let x = area
                    .x
                    .saturating_add(2)
                    .saturating_add(row_number_width)
                    .saturating_add(COLUMN_SPACING)
                    .saturating_add(
                        (visible_column as u16).saturating_mul(COLUMN_WIDTH + COLUMN_SPACING),
                    );
                let y = area.y.saturating_add(2).saturating_add(visible_row as u16);
                let copied_area =
                    Rect::new(x, y, COLUMN_WIDTH.min(area.right().saturating_sub(x)), 1);
                buf.set_style(copied_area, Style::new().reversed().green());
            }
        }
    }
}
