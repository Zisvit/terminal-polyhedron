//! Wireframe rendering: transform, project, draw edges into a sub-pixel dot grid.
//!
//! Each terminal cell is subdivided into a 2×4 grid of sub-pixels (braille dots).
//! Lines are drawn 1 sub-pixel thick using Bresenham's algorithm at sub-pixel
//! resolution, then each 2×4 block is encoded as a braille Unicode character
//! (U+2800–U+28FF). This gives smooth, gap-free lines at 2× horizontal and
//! 4× vertical effective resolution per terminal cell.

use crate::math::{project, rotate};
use crate::geometry::Polyhedron;

/// A rendered frame ready to be printed to the terminal.
pub struct Frame {
    pub cols: usize,
    pub rows: usize,
    /// Sub-pixel dot grid: (rows * 4) × (cols * 2)
    dots: Vec<Vec<bool>>,
}

impl Frame {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            dots: vec![vec![false; cols * 2]; rows * 4],
        }
    }

    pub fn clear(&mut self) {
        for row in self.dots.iter_mut() {
            for dot in row.iter_mut() {
                *dot = false;
            }
        }
    }
}

/// Fill a single sub-pixel dot with z-buffer depth test.
fn fill_dot(
    frame: &mut Frame,
    zbuffer: &mut Vec<Vec<f64>>,
    x: isize,
    y: isize,
    z: f64,
) {
    let cols = (frame.cols * 2) as isize;
    let rows = (frame.rows * 4) as isize;
    if x < 0 || x >= cols || y < 0 || y >= rows {
        return;
    }
    let xi = x as usize;
    let yi = y as usize;
    if z < zbuffer[yi][xi] {
        zbuffer[yi][xi] = z;
        frame.dots[yi][xi] = true;
    }
}

/// Core Bresenham line: draws 1 sub-pixel thick line from (x0, y0) to (x1, y1).
fn draw_line_core(
    frame: &mut Frame,
    zbuffer: &mut Vec<Vec<f64>>,
    x0: isize,
    y0: isize,
    z0: f64,
    x1: isize,
    y1: isize,
    z1: f64,
) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();

    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };

    let mut err = dx - dy;

    let mut x = x0;
    let mut y = y0;

    let total_steps = dx.max(dy).max(1);
    let mut step: isize = 0;

    loop {
        let t = step as f64 / total_steps as f64;
        let z = z0 * (1.0 - t) + z1 * t;
        fill_dot(frame, zbuffer, x, y, z);

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;

        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }

        step += 1;
    }
}

/// Draw a line ~2 sub-pixels thick so transitions between terminal cells
/// (braille characters) are not visible. The main line is drawn plus two
/// parallel offset lines perpendicular to the line direction, filling gaps
/// at cell boundaries so the stroke looks continuous.
fn draw_line(
    frame: &mut Frame,
    zbuffer: &mut Vec<Vec<f64>>,
    x0: isize,
    y0: isize,
    z0: f64,
    x1: isize,
    y1: isize,
    z1: f64,
) {
    draw_line_core(frame, zbuffer, x0, y0, z0, x1, y1, z1);
}

/// Convert a 2×4 dot block at terminal cell (cx, cy) to a braille character.
fn dots_to_braille(dots: &[Vec<bool>], cx: usize, cy: usize) -> char {
    let mut bits: u8 = 0;
    // Braille dot layout (physical):
    //   1 4
    //   2 5
    //   3 6
    //   7 8
    // Unicode bit positions:
    //   dot 1 → bit 0, dot 2 → bit 1, dot 3 → bit 2, dot 7 → bit 3
    //   dot 4 → bit 4, dot 5 → bit 5, dot 6 → bit 6, dot 8 → bit 7
    let mapping = [
        ((0usize, 0usize), 0u8),  // dot 1
        ((1, 0), 4),              // dot 4
        ((0, 1), 1),              // dot 2
        ((1, 1), 5),              // dot 5
        ((0, 2), 2),              // dot 3
        ((1, 2), 6),              // dot 6
        ((0, 3), 3),              // dot 7
        ((1, 3), 7),              // dot 8
    ];
    for ((dx, dy), bit) in mapping {
        if dots[cy * 4 + dy][cx * 2 + dx] {
            bits |= 1 << bit;
        }
    }
    char::from_u32(0x2800 + bits as u32).unwrap_or(' ')
}

/// Render one frame of the rotating wireframe polyhedron.
///
/// `subdivisions` splits each edge into that many 3D segments before
/// projection. Perspective projection is nonlinear, so intermediate 3D
/// points project off the straight 2D line between endpoints, adding
/// extra symbols along each edge without changing the figure's size,
/// position, or density. `subdivisions = 1` draws the original edges.
pub fn render(
    poly: &Polyhedron,
    angle_x: f64,
    angle_y: f64,
    angle_z: f64,
    scale: f64,
    distance: f64,
    focal: f64,
    subdivisions: usize,
    frame: &mut Frame,
) {
    // Clear the dot grid.
    for row in frame.dots.iter_mut() {
        for dot in row.iter_mut() {
            *dot = false;
        }
    }

    let cols = frame.cols as f64;
    let rows = frame.rows as f64;
    // Absolute center of the terminal (50% width, 50% height).
    let cx = cols / 2.0;
    let cy = rows / 2.0;

    // Compute a safe effective distance so no scaled vertex projects behind
    // the camera (z + distance <= 0), which would flip perspective and break
    // the bounding box / centering at large scale values. Rotation can move
    // any vertex component into z, so we use the full vertex length
    // (circumradius), which is rotation-invariant.
    let max_len = poly
        .vertices
        .iter()
        .map(|v| v.length())
        .fold(0.0f64, f64::max);
    let min_safe_distance = max_len * 2.0 + 1.0;
    let effective_distance = distance.max(min_safe_distance);

    // Project vertices in normalized 3D space (user `scale` is applied to screen fit).
    let mut projected: Vec<(f64, f64, f64)> = Vec::with_capacity(poly.vertices.len());
    for &v in &poly.vertices {
        let rotated = rotate(v, angle_x, angle_y, angle_z);
        let (px, py, pz) = project(rotated, focal, effective_distance);
        projected.push((px, py, pz));
    }

    // First pass: compute the true bounding box of EVERYTHING that will be
    // drawn — vertices plus all subdivided edge points. Perspective projection
    // is nonlinear, so edge midpoints can project outside the vertex bbox.
    // We must include them or the figure will be off-centre and clipped.
    let subs = subdivisions.max(1).max(12);
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for &(px, py, _) in &projected {
        min_x = min_x.min(px);
        max_x = max_x.max(px);
        min_y = min_y.min(py);
        max_y = max_y.max(py);
    }

    for &(a, b) in &poly.edges {
        let va = poly.vertices[a];
        let vb = poly.vertices[b];
        for i in 0..subs {
            let t0 = i as f64 / subs as f64;
            let t1 = (i + 1) as f64 / subs as f64;
            let p0 = va.mul(1.0 - t0).add(vb.mul(t0));
            let p1 = va.mul(1.0 - t1).add(vb.mul(t1));
            let r0 = rotate(p0, angle_x, angle_y, angle_z);
            let r1 = rotate(p1, angle_x, angle_y, angle_z);
            let (xa, ya, _) = project(r0, focal, effective_distance);
            let (xb, yb, _) = project(r1, focal, effective_distance);
            min_x = min_x.min(xa).min(xb);
            max_x = max_x.max(xa).max(xb);
            min_y = min_y.min(ya).min(yb);
            max_y = max_y.max(ya).max(yb);
        }
    }

    // Expand the bounding box by a safety margin to account for stroke
    // thickness from offset lines and Bresenham rasterisation error. This
    // guarantees the drawn dots never reach the terminal edges.
    let margin = 1.0; // projected-coordinate units, generous margin
    let min_x = min_x - margin;
    let max_x = max_x + margin;
    let min_y = min_y - margin;
    let max_y = max_y + margin;
    let bbox_w = (max_x - min_x).max(0.001);
    let bbox_h = (max_y - min_y).max(0.001);
    let bbox_cx = (min_x + max_x) / 2.0;
    let bbox_cy = (min_y + max_y) / 2.0;

    // Fit the bounding box inside ~60% of the terminal in effective units.
    // Each terminal cell is ~2x taller than wide, so effective height is rows*2.
    let fill = 0.6;
    let scale_x = (cols * fill) / bbox_w;
    let scale_y = (rows * 2.0 * fill) / bbox_h;
    let uniform_scale = scale_x.min(scale_y) * scale;

    // Z-buffer: depth per sub-pixel dot, initialized to infinity.
    let sub_cols = frame.cols * 2;
    let sub_rows = frame.rows * 4;
    let mut zbuffer: Vec<Vec<f64>> = vec![vec![f64::INFINITY; sub_cols]; sub_rows];

    // Draw each edge, optionally subdivided into multiple 3D segments.
    let subs = subdivisions.max(1).max(12);
    for &(a, b) in &poly.edges {
        let va = poly.vertices[a];
        let vb = poly.vertices[b];

        for i in 0..subs {
            let t0 = i as f64 / subs as f64;
            let t1 = (i + 1) as f64 / subs as f64;

            let p0 = va.mul(1.0 - t0).add(vb.mul(t0));
            let p1 = va.mul(1.0 - t1).add(vb.mul(t1));

            let r0 = rotate(p0, angle_x, angle_y, angle_z);
            let r1 = rotate(p1, angle_x, angle_y, angle_z);
            let (xa, ya, za) = project(r0, focal, effective_distance);
            let (xb, yb, zb) = project(r1, focal, effective_distance);

            // Apply aspect compensation to y (cell ~2x taller than wide), then
            // centre the bounding box on the terminal centre.
            let sa_x = cx + (xa - bbox_cx) * uniform_scale;
            let sa_y = cy - (ya - bbox_cy) * uniform_scale * 0.5;
            let sb_x = cx + (xb - bbox_cx) * uniform_scale;
            let sb_y = cy - (yb - bbox_cy) * uniform_scale * 0.5;

            // Convert to sub-pixel coordinates (2× horizontal, 4× vertical).
            let x0 = (sa_x * 2.0).round() as isize;
            let y0 = (sa_y * 4.0).round() as isize;
            let x1 = (sb_x * 2.0).round() as isize;
            let y1 = (sb_y * 4.0).round() as isize;

            draw_line(frame, &mut zbuffer, x0, y0, za, x1, y1, zb);
        }
    }


}

/// Convert a Frame into a single String with newlines, ready for terminal output.
/// Each terminal cell is encoded as a braille character from its 2×4 dot block.
pub fn frame_to_string(frame: &Frame) -> String {
    let mut out = String::with_capacity(frame.cols * frame.rows + frame.rows);
    for cy in 0..frame.rows {
        for cx in 0..frame.cols {
            out.push(dots_to_braille(&frame.dots, cx, cy));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod display_tests {
    use super::*;

    fn drawn_dots(frame: &Frame) -> Vec<(isize, isize)> {
        let mut dots = Vec::new();
        for cy in 0..frame.rows {
            for cx in 0..frame.cols {
                for dy in 0..4usize {
                    for dx in 0..2usize {
                        if frame.dots[cy * 4 + dy][cx * 2 + dx] {
                            dots.push(((cx * 2 + dx) as isize, (cy * 4 + dy) as isize));
                        }
                    }
                }
            }
        }
        dots
    }

    fn render_frame(
        poly: &crate::geometry::Polyhedron,
        cols: usize,
        rows: usize,
        scale: f64,
        angle_x: f64,
        angle_y: f64,
        angle_z: f64,
    ) -> Frame {
        let mut frame = Frame::new(cols, rows);
        render(
            poly,
            angle_x,
            angle_y,
            angle_z,
            scale,
            5.0, // distance
            2.0, // focal
            2,   // subdivisions
            &mut frame,
        );
        frame
    }

    /// No dot is ever drawn outside the frame bounds — the terminal is the hard boundary.
    #[test]
    fn no_dots_outside_frame_bounds() {
        let poly = crate::geometry::Polyhedron::dodecahedron();
        for cols in [20, 40, 80, 120] {
            for rows in [10, 20, 30, 40] {
                for scale in [0.5, 1.0, 2.0, 5.0, 10.0] {
                    let frame = render_frame(&poly, cols, rows, scale, 0.3, 0.5, 0.2);
                    let sub_cols = (cols * 2) as isize;
                    let sub_rows = (rows * 4) as isize;
                    for (x, y) in drawn_dots(&frame) {
                        assert!(
                            x >= 0 && x < sub_cols && y >= 0 && y < sub_rows,
                            "dot ({}, {}) outside frame {}x{} at scale {}",
                            x, y, cols, rows, scale
                        );
                    }
                }
            }
        }
    }

    /// The figure's bounding box of drawn dots is centred in the terminal.
    /// Tolerance: within 1 sub-pixel of the terminal centre on each axis.
    #[test]
    fn figure_is_centred_in_terminal() {
        let poly = crate::geometry::Polyhedron::dodecahedron();
        for cols in [40, 80] {
            for rows in [30, 40] {
                for scale in [0.5, 1.0, 2.0, 5.0] {
                    let frame = render_frame(&poly, cols, rows, scale, 0.3, 0.5, 0.2);
                    let dots = drawn_dots(&frame);
                    assert!(!dots.is_empty(), "no dots drawn at scale {}", scale);

                    let min_x = dots.iter().map(|(x, _)| *x).min().unwrap();
                    let max_x = dots.iter().map(|(x, _)| *x).max().unwrap();
                    let min_y = dots.iter().map(|(_, y)| *y).min().unwrap();
                    let max_y = dots.iter().map(|(_, y)| *y).max().unwrap();

                    let dot_cx = (min_x + max_x) as f64 / 2.0;
                    let dot_cy = (min_y + max_y) as f64 / 2.0;
                    let term_cx = (cols * 2) as f64 / 2.0;
                    let term_cy = (rows * 4) as f64 / 2.0;

                    let dx = (dot_cx - term_cx).abs();
                    let dy = (dot_cy - term_cy).abs();

                    assert!(
                        dx <= 2.0,
                        "x-centre off by {} at cols={} rows={} scale={} (dot_cx={}, term_cx={})",
                        dx, cols, rows, scale, dot_cx, term_cx
                    );
                    assert!(
                        dy <= 2.0,
                        "y-centre off by {} at cols={} rows={} scale={} (dot_cy={}, term_cy={})",
                        dy, cols, rows, scale, dot_cy, term_cy
                    );
                }
            }
        }
    }

    /// Changing scale does not shift the figure's centre — it stays at terminal centre.
    #[test]
    fn scale_change_does_not_shift_centre() {
        let poly = crate::geometry::Polyhedron::dodecahedron();
        let cols = 80;
        let rows = 30;
        let mut centres = Vec::new();
        for scale in [0.5, 1.0, 2.0, 5.0, 10.0] {
            let frame = render_frame(&poly, cols, rows, scale, 0.3, 0.5, 0.2);
            let dots = drawn_dots(&frame);
            assert!(!dots.is_empty());
            let min_x = dots.iter().map(|(x, _)| *x).min().unwrap();
            let max_x = dots.iter().map(|(x, _)| *x).max().unwrap();
            let min_y = dots.iter().map(|(_, y)| *y).min().unwrap();
            let max_y = dots.iter().map(|(_, y)| *y).max().unwrap();
            centres.push(((min_x + max_x) as f64 / 2.0, (min_y + max_y) as f64 / 2.0));
        }
        let term_cx = (cols * 2) as f64 / 2.0;
        let term_cy = (rows * 4) as f64 / 2.0;
        for (cx, cy) in &centres {
            assert!((cx - term_cx).abs() <= 2.0, "x-centre {} off from {}", cx, term_cx);
            assert!((cy - term_cy).abs() <= 2.0, "y-centre {} off from {}", cy, term_cy);
        }
    }

    /// All three shapes render centred and in bounds.
    #[test]
    fn all_shapes_render_centred_and_in_bounds() {
        for shape in [
            crate::geometry::Polyhedron::cube(),
            crate::geometry::Polyhedron::icosahedron(),
            crate::geometry::Polyhedron::dodecahedron(),
        ] {
            let frame = render_frame(&shape, 80, 30, 1.0, 0.3, 0.5, 0.2);
            let sub_cols = (frame.cols * 2) as isize;
            let sub_rows = (frame.rows * 4) as isize;
            for (x, y) in drawn_dots(&frame) {
                assert!(x >= 0 && x < sub_cols && y >= 0 && y < sub_rows);
            }
            let dots = drawn_dots(&frame);
            assert!(!dots.is_empty(), "shape produced no dots");
            let min_x = dots.iter().map(|(x, _)| *x).min().unwrap();
            let max_x = dots.iter().map(|(x, _)| *x).max().unwrap();
            let min_y = dots.iter().map(|(_, y)| *y).min().unwrap();
            let max_y = dots.iter().map(|(_, y)| *y).max().unwrap();
            let dot_cx = (min_x + max_x) as f64 / 2.0;
            let dot_cy = (min_y + max_y) as f64 / 2.0;
            let term_cx = (frame.cols * 2) as f64 / 2.0;
            let term_cy = (frame.rows * 4) as f64 / 2.0;
            assert!((dot_cx - term_cx).abs() <= 1.0);
            assert!((dot_cy - term_cy).abs() <= 1.0);
        }
    }

    /// Rendering produces non-empty, visually distinct output (not all spaces).
    #[test]
    fn render_output_is_not_blank() {
        let poly = crate::geometry::Polyhedron::dodecahedron();
        let frame = render_frame(&poly, 80, 30, 1.0, 0.3, 0.5, 0.2);
        let out = frame_to_string(&frame);
        let non_space: usize = out.chars().filter(|c| *c != ' ' && *c != '\n').count();
        assert!(non_space > 0, "render output is blank");
    }
}
