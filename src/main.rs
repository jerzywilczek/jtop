use clap::Parser;
use jwtop::app::{App, AppResult};
use jwtop::event::{Event, EventHandler};
use jwtop::handler::handle_key_events;
use jwtop::tui::Tui;
use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

const TICK_RATE: u64 = 1000;

fn main() -> AppResult<()> {
    let cli = jwtop::config::Cli::parse();

    if cli.dump_config {
        println!("{}", jwtop::config::sample_config());
        return Ok(());
    }

    if cli.dump_sample_theme {
        println!("{}", jwtop::config::sample_theme());
        return Ok(());
    }

    // TODO: load the config and actually use it in the app.

    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(TICK_RATE);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next()? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
