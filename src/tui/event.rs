use crossterm::event::{self, Event, KeyEvent};
use std::time::Duration;

pub enum AppEvent {
    Key(KeyEvent),
    Resize(u16, u16),
    Tick,
}

pub struct EventReader;

impl EventReader {
    /// Poll for the next event with timeout (non-blocking for async).
    pub fn poll(timeout: Duration) -> std::io::Result<Option<AppEvent>> {
        if event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => Ok(Some(AppEvent::Key(key))),
                Event::Resize(w, h) => Ok(Some(AppEvent::Resize(w, h))),
                _ => Ok(None),
            }
        } else {
            Ok(Some(AppEvent::Tick))
        }
    }
}
