//! Entry point: parse CLI args and run the polyhedron viewer.

use clap::Parser;
use terminal_polyhedron::app::{App, Config};

#[derive(Parser, Debug)]
#[command(author, version, about = "Rotating 3D polyhedron in the terminal", long_about = None)]
struct Args {
    /// Shape to render: cube, icosahedron, or dodecahedron.
    #[arg(long, default_value = "dodecahedron")]
    shape: String,

    /// Target frames per second.
    #[arg(long, default_value = "120")]
    fps: u64,

    /// Scale of the polyhedron.
    #[arg(long, default_value = "1.0")]
    scale: f64,

    /// Distance from camera to object along Z.
    #[arg(long, default_value = "5.0")]
    distance: f64,

    /// Focal length for perspective projection.
    #[arg(long, default_value = "2.0")]
    focal: f64,

    /// Number of segments per edge (more = more symbols, same figure size).
    #[arg(long, default_value = "2")]
    subdivisions: usize,

    /// Auto-rotation speed in radians per second.
    #[arg(long, default_value = "0.2")]
    speed: f64,
    /// Render a single frame to stdout (no TTY/raw mode) for debugging.
    #[arg(long)]
    dump: bool,

    /// Terminal columns for --dump (defaults to $COLUMNS or 80).
    #[arg(long)]
    cols: Option<u16>,

    /// Terminal rows for --dump (defaults to $LINES or 24).
    #[arg(long)]
    rows: Option<u16>,
}

fn main() {
    let args = Args::parse();

    let config = Config {
        shape: args.shape,
        fps: args.fps,
        scale: args.scale,
        distance: args.distance,
        focal: args.focal,
        subdivisions: args.subdivisions,
        auto_rotate_speed: args.speed,
        ..Default::default()
    };

    if args.dump {
        use terminal_polyhedron::geometry::Polyhedron;
        use terminal_polyhedron::render::{render, frame_to_string, Frame};
        use terminal_polyhedron::math::deg_to_rad;
        let poly = match Polyhedron::by_name(&config.shape) {
            Some(p) => p,
            None => {
                eprintln!("unknown shape: {}", config.shape);
                std::process::exit(1);
            }
        };
        let cols = args.cols.or_else(|| std::env::var("COLUMNS").ok().and_then(|v| v.parse().ok())).unwrap_or(80) as usize;
        let rows = args.rows.or_else(|| std::env::var("LINES").ok().and_then(|v| v.parse().ok())).unwrap_or(24) as usize;
        let mut frame = Frame::new(cols, rows);
        render(&poly, deg_to_rad(15.0), deg_to_rad(25.0), deg_to_rad(10.0), config.scale, config.distance, config.focal, config.subdivisions, &mut frame);
        print!("{}", frame_to_string(&frame));
        return;
    }

    match App::new(config) {
        Ok(app) => {
            if let Err(e) = app.run() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize terminal: {}", e);
            std::process::exit(1);
        }
    }
}
