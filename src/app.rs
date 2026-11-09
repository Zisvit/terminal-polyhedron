//! Application loop: smooth auto-rotation, manual rotation, event handling.

use crate::geometry::Polyhedron;
use crate::math::deg_to_rad;
use crate::render::{render, frame_to_string, Frame};
use crate::terminal::{Terminal, TerminalSize};
use crossterm::event::{Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::execute;
use std::io::{Result as IoResult, Write};
use std::time::{Duration, Instant};

/// Configuration for the animation.
pub struct Config {
    pub shape: String,
    pub fps: u64,
    pub scale: f64,
    pub distance: f64,
    pub focal: f64,
    pub subdivisions: usize,
    pub auto_rotate_speed: f64, // radians per second
    pub manual_step: f64,       // radians per key press
}

impl Default for Config {
    fn default() -> Self {
        Self {
            shape: "dodecahedron".into(),
            fps: 120,
            scale: 1.0,
            distance: 5.0,
            focal: 2.0,
            subdivisions: 12,
            auto_rotate_speed: 0.8,
            manual_step: deg_to_rad(10.0),
        }
    }
}

/// Application state.
pub struct App {
    pub config: Config,
    pub poly: Polyhedron,
    pub angle_x: f64,
    pub angle_y: f64,
    pub angle_z: f64,
    pub running: bool,
    pub last_frame: Instant,
    pub frame_interval: Duration,
    pub terminal_size: TerminalSize,
    pub frame: Frame,
    pub drag_last: Option<(u16, u16)>,
    pub drag_button: Option<MouseButton>,
}

impl App {
    pub fn new(config: Config) -> IoResult<Self> {
        let poly = Polyhedron::by_name(&config.shape)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("unknown shape: {}", config.shape)))?;

        let terminal = Terminal::new()?;
        let size = terminal.size();
        let frame_interval = Duration::from_secs_f64(1.0 / config.fps as f64);

        Ok(Self {
            config,
            poly,
            angle_x: deg_to_rad(15.0),
            angle_y: deg_to_rad(25.0),
            angle_z: deg_to_rad(10.0),
            running: true,
            last_frame: Instant::now(),
            frame_interval,
            terminal_size: size,
            frame: Frame::new(size.cols as usize, size.rows as usize),
            drag_last: None,
            drag_button: None,
        })
    }

    /// Handle a terminal event. Returns true if the app should quit.
    pub fn handle_event(&mut self, event: Event) -> bool {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return true,
                    KeyCode::Char('r') => {
                        self.angle_x = 0.0;
                        self.angle_y = 0.0;
                        self.angle_z = 0.0;
                    }
                    KeyCode::Left => {
                        self.angle_y -= self.config.manual_step;
                    }
                    KeyCode::Right => {
                        self.angle_y += self.config.manual_step;
                    }
                    KeyCode::Up => {
                        self.angle_x -= self.config.manual_step;
                    }
                    KeyCode::Down => {
                        self.angle_x += self.config.manual_step;
                    }
                    KeyCode::Char('a') => {
                        self.angle_z -= self.config.manual_step;
                    }
                    KeyCode::Char('d') => {
                        self.angle_z += self.config.manual_step;
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') => {
                        self.config.scale *= 1.1;
                    }
                    KeyCode::Char('-') => {
                        self.config.scale *= 0.9;
                    }
                    _ => {}
                }
            }
            Event::Mouse(m) => match m.kind {
                MouseEventKind::Down(btn @ (MouseButton::Left | MouseButton::Right)) => {
                    self.drag_last = Some((m.column, m.row));
                    self.drag_button = Some(btn);
                }
                MouseEventKind::Drag(btn @ (MouseButton::Left | MouseButton::Right)) => {
                    if self.drag_button == Some(btn) {
                        if let Some((lx, ly)) = self.drag_last {
                            let dx = m.column as f64 - lx as f64;
                            let dy = m.row as f64 - ly as f64;
                            let s = 0.03;
                            match btn {
                                MouseButton::Left => {
                                    self.angle_y += dx * s;
                                    self.angle_x += dy * s;
                                }
                                MouseButton::Right => {
                                    self.angle_z += dx * s;
                                }
                                _ => {}
                            }
                        }
                    }
                    self.drag_last = Some((m.column, m.row));
                    self.drag_button = Some(btn);
                }
                MouseEventKind::Up(_) => {
                    self.drag_last = None;
                    self.drag_button = None;
                }
                MouseEventKind::ScrollUp => {
                    self.config.scale *= 1.1;
                }
                MouseEventKind::ScrollDown => {
                    self.config.scale *= 0.9;
                }
                _ => {}
            },
            Event::Resize(cols, rows) => {
                self.terminal_size = TerminalSize { cols, rows };
                self.frame = Frame::new(cols as usize, rows as usize);
            }
            _ => {}
        }
        false
    }

    /// Update rotation angles based on elapsed time (auto-rotation).
    pub fn update(&mut self, dt: f64) {
        let dt = dt.min(0.05);
        let step = self.config.auto_rotate_speed * dt;
        self.angle_x += step * 0.5;
        self.angle_y += step;
        self.angle_z += step * 0.25;
    }

    /// Render one frame to the terminal.
    pub fn draw(&mut self, terminal: &mut Terminal) -> IoResult<()> {
        render(
            &self.poly,
            self.angle_x,
            self.angle_y,
            self.angle_z,
            self.config.scale,
            self.config.distance,
            self.config.focal,
            self.config.subdivisions,
            &mut self.frame,
        );

        let out = frame_to_string(&self.frame);
        let mut buf = String::with_capacity(out.len() + self.frame.rows * 8);
        use crossterm::cursor::MoveTo;
        for (row, line) in out.lines().enumerate() {
            buf.push_str(&MoveTo(0, row as u16).to_string());
            buf.push_str(line);
        }
        execute!(terminal.stdout, crossterm::cursor::MoveTo(0, 0))?;
        write!(terminal.stdout, "{}", buf)?;
        terminal.flush()?;
        Ok(())
    }

    /// Main run loop.
    pub fn run(mut self) -> IoResult<()> {
        let mut terminal = Terminal::new()?;

        while self.running {
            let now = Instant::now();
            let dt = now.duration_since(self.last_frame).as_secs_f64();
            self.last_frame = now;

            // Track terminal size each frame so resize is handled even if
            // the event is missed. Recreate the frame when dimensions change.
            let size = terminal.size();
            if size.cols != self.terminal_size.cols || size.rows != self.terminal_size.rows {
                self.terminal_size = size;
                self.frame = Frame::new(size.cols as usize, size.rows as usize);
            }

            // Auto-rotate.
            self.update(dt);

            // Poll for events (non-blocking).
            if let Ok(Some(event)) = terminal.poll_event(0) {
                if self.handle_event(event) {
                    break;
                }
            }

            // Draw.
            let _ = self.draw(&mut terminal);

            // Frame pacing: sleep to hit target FPS.
            let elapsed = now.elapsed();
            if elapsed < self.frame_interval {
                std::thread::sleep(self.frame_interval - elapsed);
            }
        }

        Ok(())
    }
}
