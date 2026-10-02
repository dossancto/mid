use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::core::query::TableCommand;

#[derive(Clone, Copy)]
pub enum KeybindEvents {
    NextRow,
    PreviousRow,
    NextColumn,
    PreviousColumn,
    FirstRow,
    LastRow,
    YankSelection,
    UpdateSelection,
    EditQuery,
    TableSearch,
    GoToRow,
    Filter,
    OpenValue,
    Quit,
    SelectMode,
    OpenValueInSelectMode,
    SortByColumn,
}

impl KeybindEvents {
    pub fn parse_to_event(input: &KeyEvent, command: &TableCommand) -> Option<Self> {
        match input.code {
            KeyCode::Up | KeyCode::Char('k') => Some(Self::PreviousRow),
            KeyCode::Down | KeyCode::Char('j') => Some(Self::NextRow),
            KeyCode::Left | KeyCode::Char('h') => Some(Self::PreviousColumn),
            KeyCode::Right | KeyCode::Char('l') => Some(Self::NextColumn),
            KeyCode::Char('g') if input.modifiers.contains(KeyModifiers::SHIFT) => {
                Some(Self::GoToRow)
            }
            KeyCode::Home | KeyCode::Char('g') => Some(Self::FirstRow),
            KeyCode::End | KeyCode::Char('G') => Some(Self::LastRow),
            KeyCode::Char('y') => Some(Self::YankSelection),
            KeyCode::Char('s') => Some(Self::SortByColumn),
            KeyCode::Char('q') => Some(Self::Quit),
            KeyCode::Char('u') => Some(Self::UpdateSelection),
            KeyCode::Char('e') => Some(Self::EditQuery),
            KeyCode::Enter | KeyCode::Char(' ')
                if input.modifiers.contains(KeyModifiers::SHIFT) =>
            {
                Some(Self::OpenValueInSelectMode)
            }
            KeyCode::Enter | KeyCode::Char(' ') => match command {
                TableCommand::ShowTables => Some(Self::TableSearch),
                TableCommand::ShowValue => Some(Self::OpenValue),
            },
            KeyCode::Char('f') => Some(Self::Filter),
            KeyCode::Char('o') => Some(Self::OpenValue),
            KeyCode::Char('v') => Some(Self::SelectMode),
            _ => None,
        }
    }

    pub fn parse_to_command(&self) -> &'static str {
        match self {
            Self::NextRow => "k",
            Self::PreviousRow => "j",
            Self::NextColumn => "l",
            Self::PreviousColumn => "h",
            Self::SortByColumn => "s",
            Self::FirstRow => "g",
            Self::LastRow => "G",
            Self::YankSelection => "y",
            Self::UpdateSelection => "u",
            Self::EditQuery => "e",
            Self::TableSearch => "Enter",
            Self::Filter => "f",
            Self::OpenValue => "Enter",
            Self::GoToRow => "Shift+g",
            Self::Quit => "q",
            Self::SelectMode => "v",
            Self::OpenValueInSelectMode => "Shift+Enter",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::NextRow => "up",
            Self::PreviousRow => "down",
            Self::NextColumn => "next column",
            Self::PreviousColumn => "previous column",
            Self::FirstRow => "first row",
            Self::SortByColumn => "sort by column",
            Self::LastRow => "last row",
            Self::YankSelection => "yank",
            Self::UpdateSelection => "update",
            Self::EditQuery => "edit",
            Self::TableSearch => "select",
            Self::GoToRow => "go to",
            Self::Filter => "filter",
            Self::OpenValue => "value",
            Self::Quit => "quit",
            Self::SelectMode => "select mode",
            Self::OpenValueInSelectMode => "open value in select mode",
        }
    }

    pub fn footer_label(&self, select_mode: bool) -> &'static str {
        match (self, select_mode) {
            (Self::OpenValue, true) => "select value",
            (Self::SelectMode, true) => "exit select mode",
            _ => self.label(),
        }
    }

    pub fn footer_events(command: &TableCommand, select_mode: bool) -> &'static [Self] {
        const TABLE_EVENTS: &[KeybindEvents] = &[
            KeybindEvents::GoToRow,
            KeybindEvents::Filter,
            KeybindEvents::TableSearch,
            KeybindEvents::YankSelection,
            KeybindEvents::EditQuery,
            KeybindEvents::Quit,
        ];
        const VALUE_EVENTS: &[KeybindEvents] = &[
            KeybindEvents::GoToRow,
            KeybindEvents::Filter,
            KeybindEvents::OpenValue,
            KeybindEvents::YankSelection,
            KeybindEvents::UpdateSelection,
            KeybindEvents::SelectMode,
            KeybindEvents::EditQuery,
            KeybindEvents::Quit,
        ];
        const SELECT_EVENTS: &[KeybindEvents] = &[
            KeybindEvents::FirstRow,
            KeybindEvents::LastRow,
            KeybindEvents::OpenValue,
            KeybindEvents::UpdateSelection,
            KeybindEvents::SelectMode,
            KeybindEvents::Quit,
        ];

        if select_mode {
            return SELECT_EVENTS;
        }

        match command {
            TableCommand::ShowTables => TABLE_EVENTS,
            TableCommand::ShowValue => VALUE_EVENTS,
        }
    }
}
