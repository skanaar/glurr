use std::{io, panic};
use ratatui::crossterm::cursor::Show;
use ratatui::crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal;
use ratatui::crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};

pub struct Tui {}

impl Tui {
    pub fn enter() -> std::io::Result<()> {
        terminal::enable_raw_mode()?;
        execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;
        // Define a custom panic hook to reset the terminal properties.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::teardown().expect("failed to reset the terminal");
            panic_hook(panic);
        }));
        Ok(())
    }

    pub fn teardown() -> std::io::Result<()> {
        terminal::disable_raw_mode()?;
        execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture, Show)?;
        Ok(())
    }
}
