//! Terminal handling: alternate screen, raw mode, cursor, resize events.

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{self, size};
use crossterm::cursor::{Hide, Show};
use std::io::{stdout, Write, Result as IoResult};

/// Size of the terminal in columns and rows.
#[derive(Clone, Copy, Debug)]
pub struct TerminalSize {
    pub cols: u16,
    pub rows: u16,
}

/// RAII wrapper that puts the terminal into alternate screen / raw mode
/// and restores it on drop.
pub struct Terminal {
    pub stdout: std::io::Stdout,
}

impl Terminal {
    pub fn new() -> IoResult<Self> {
        let mut stdout = stdout();
        execute!(
            stdout,
            terminal::EnterAlternateScreen,
            Hide,
            event::EnableMouseCapture
        )?;
        terminal::enable_raw_mode()?;
        Ok(Self { stdout })
    }

    pub fn size(&self) -> TerminalSize {
        let (cols, rows) = size().unwrap_or((80, 24));
        TerminalSize { cols, rows }
    }

    pub fn flush(&mut self) -> IoResult<()> {
        self.stdout.flush()
    }

    pub fn poll_event(&mut self, timeout_ms: u64) -> IoResult<Option<Event>> {
        if event::poll(std::time::Duration::from_millis(timeout_ms))? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = execute!(
            self.stdout,
            event::DisableMouseCapture,
            Show,
            terminal::LeaveAlternateScreen
        );
        let _ = terminal::disable_raw_mode();
    }
}
