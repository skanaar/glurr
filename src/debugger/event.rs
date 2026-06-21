use std::time::{Duration, Instant};
use ratatui::crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
}

const POLL_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Debug)]
pub struct Events {
    last_tick: Instant,
}

impl Events {
    pub fn new() -> Self { Events { last_tick: Instant::now() } }

    /// Block until the next event is available, returning a [`Event::Tick`]
    /// if `POLL_INTERVAL` elapses first.
    pub fn next(&mut self) -> std::io::Result<Event> {
        loop {
            let timeout = POLL_INTERVAL
                .checked_sub(self.last_tick.elapsed())
                .unwrap_or(POLL_INTERVAL);

            if event::poll(timeout)? {
                match event::read()? {
                    CrosstermEvent::Key(e) => {
                        if e.kind == event::KeyEventKind::Press {
                            return Ok(Event::Key(e));
                        }
                        // ignore KeyEventKind::Release on windows
                    }
                    _ => {}
                }
            }

            if self.last_tick.elapsed() >= POLL_INTERVAL {
                self.last_tick = Instant::now();
                return Ok(Event::Tick);
            }
        }
    }
}
