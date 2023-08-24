use crate::{
    app::{App, AppResult, InputState},
    ui::processes::Column,
};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

fn change_processes_sort_into(app: &mut App, selected_column: Column) {
    if let InputState::ProcessesSortSelection { column, direction } = &mut app.input_state {
        if *column == selected_column {
            *direction = direction.reversed();
        } else {
            *column = selected_column;
            *direction = column.default_sort_direction();
        }
    }
}

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
                return Ok(());
            }
        }

        _ => {}
    }

    match &mut app.input_state {
        InputState::ProcessesSortSelection { column, direction } => match key_event.code {
            KeyCode::Char('/') => {
                app.input_state = InputState::ProcessesSearch {
                    old_column: Some(*column),
                    old_direction: Some(*direction),
                    search: String::new(),
                }
            }

            KeyCode::Char('q') => {
                app.quit();
            }

            KeyCode::Char('c') | KeyCode::Char('C') => {
                change_processes_sort_into(app, Column::Cpu);
            }

            KeyCode::Char('p') | KeyCode::Char('P') => {
                change_processes_sort_into(app, Column::Pid);
            }

            KeyCode::Char('n') | KeyCode::Char('N') => {
                change_processes_sort_into(app, Column::Name);
            }

            KeyCode::Char('m') | KeyCode::Char('M') => {
                change_processes_sort_into(app, Column::Memory);
            }

            KeyCode::Char('r') | KeyCode::Char('R') => {
                change_processes_sort_into(app, Column::DiskRead);
            }

            KeyCode::Char('w') | KeyCode::Char('W') => {
                change_processes_sort_into(app, Column::DiskWrite);
            }

            // Other handlers you could add here.
            _ => {}
        },

        InputState::ProcessesSearch {
            old_column,
            old_direction,
            search,
        } => {
            match key_event.code {
                KeyCode::Esc => {
                    let column = old_column.unwrap_or_default();
                    let direction = old_direction.unwrap_or(column.default_sort_direction());
                    app.input_state = InputState::ProcessesSortSelection { column, direction }
                }

                // ctrl + backspace sends ctrl + w for some reason
                KeyCode::Char('w') | KeyCode::Char('W')
                    if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    search.clear();
                }

                KeyCode::Char(c)
                    if key_event.kind == KeyEventKind::Press
                        || key_event.kind == KeyEventKind::Repeat =>
                {
                    search.push(c);
                }

                KeyCode::Backspace
                    if key_event.kind == KeyEventKind::Press
                        || key_event.kind == KeyEventKind::Repeat =>
                {
                    search.pop();
                }

                _ => {}
            }
        }
    }

    Ok(())
}
