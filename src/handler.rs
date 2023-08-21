use crate::{
    app::{App, AppResult},
    ui::processes::Column,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn change_processes_sort_into(app: &mut App, column: Column) {
    if app.processes_sort_column == column {
        app.processes_sort_direction = app.processes_sort_direction.reversed();
    } else {
        app.processes_sort_column = column;
        app.processes_sort_direction = column.default_sort_direction();
    }
}

/// Handles the key events and updates the state of [`App`].
pub fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            } else {
                change_processes_sort_into(app, Column::Cpu)
            }
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
    }
    Ok(())
}
