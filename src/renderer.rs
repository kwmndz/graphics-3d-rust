use std::io::Write;

use crossterm::{
    cursor, queue,
    style::{self, Color, Stylize},
};

// represents the value at which a pixel is trying to be drawn to close
// aka the cutoff for drawing pixels (in the z-axis)
pub const NEAR: f32 = 0.1;

// convert 3d point to 2d point, but also return z, for z-buffering
pub fn convert_to_screen(v: &crate::math::Vector, w: f32, h: f32) -> Option<(u16, u16, f32)> {
    let [x, y, z] = v.0;
    let scale = 20.0;

    // any z closer than near should be skipped
    // to avoid divide by 0
    // z == 0 represents camera/observer
    if z < NEAR {
        return None;
    }
    let ooz = 1.0 / z;

    let sx = x * ooz * scale + w * 0.5;
    let sy = h * 0.5 - y * ooz * scale;

    if sx >= 0.0 && sx < w && sy >= 0.0 && sy < h {
        Some((sx as u16, sy as u16, z))
    } else {
        None
    }
}

// deprecated
pub fn draw_line(
    stdout: &mut std::io::Stdout,
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
) -> std::io::Result<()> {
    let x0 = x0 as f32;
    let y0 = y0 as f32;
    let x1 = x1 as f32;
    let y1 = y1 as f32;

    let dx = x1 - x0;
    let dy = y1 - y0;

    let steps = dx.abs().max(dy.abs()) as usize;

    if steps == 0 {
        queue!(
            stdout,
            cursor::MoveTo(x0 as u16, y0 as u16),
            style::PrintStyledContent("█".red())
        )?;
        return Ok(());
    }

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = x0 + dx * t;
        let y = y0 + dy * t;

        queue!(
            stdout,
            cursor::MoveTo(x.round() as u16, y.round() as u16),
            style::PrintStyledContent("@".magenta())
        )?;
    }

    Ok(())
}

fn sign(p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) -> f32 {
    (p1.0 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 - p3.1)
}

// computes if pt is in triangel made up of 3 vertexes
fn in_triangle(pt: (f32, f32), v1: (f32, f32), v2: (f32, f32), v3: (f32, f32)) -> bool {
    let d1: f32 = sign(pt, v1, v2);
    let d2: f32 = sign(pt, v2, v3);
    let d3: f32 = sign(pt, v3, v1);

    let has_neg: bool = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos: bool = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)
}

// Writes pixels into a frame buffer and depth buffer
// no terminal I/O calls all rendering is in the flush_frame fn
pub fn draw_face(
    frame_buf: &mut [Option<Color>],
    projected: &[Option<(u16, u16, f32)>; 8],
    c: (usize, usize, usize),
    color: Color,
    depth_map: &mut Vec<f32>,
    width: u16,
) {
    if let (Some((x0, y0, z0)), Some((x1, y1, z1)), Some((x2, y2, z2))) =
        (projected[c.0], projected[c.1], projected[c.2])
    {
        let x0 = x0 as f32;
        let y0 = y0 as f32;

        let x1 = x1 as f32;
        let y1 = y1 as f32;

        let x2 = x2 as f32;
        let y2 = y2 as f32;

        // back-face culling: signed area of projected triangle
        // negative = front-facing (CW in screen y-down), non-negative = back-facing 
        let total_area = sign((x0, y0), (x1, y1), (x2, y2));
        if total_area >= 0.0 {
            return;
        }

        let minx = x2.min(x0.min(x1));
        let miny = y2.min(y0.min(y1));
        let maxx = x2.max(x0.max(x1));
        let maxy = y2.max(y0.max(y1));

        let dx = maxx - minx;
        let dy = maxy - miny;

        let steps = dx.abs() as usize;
        let steps1 = dy.abs() as usize;

        for i in 0..=steps {
            for i1 in 0..=steps1 {
                let t = if steps == 0 { 0.0 } else { i as f32 / steps as f32 };
                let t1 = if steps1 == 0 { 0.0 } else { i1 as f32 / steps1 as f32 };
                let x = minx + dx * t;
                let y = miny + dy * t1;

                if !in_triangle((x, y), (x0, y0), (x1, y1), (x2, y2)) {
                    continue;
                }

                // get the z value of each pixel
                // based off the weighted value of the original 3 z-vals
                let w0 = sign((x, y), (x1, y1), (x2, y2)) / total_area;
                let w1 = sign((x0, y0), (x, y), (x2, y2)) / total_area;
                let w2 = sign((x0, y0), (x1, y1), (x, y)) / total_area;

                // w0+w1+w2 == 1 for all points inside triangle
                let pixel_z = w0 * z0 + w1 * z1 + w2 * z2;

                let idx = y.round() as usize * width as usize + x.round() as usize;

                // dont draw if a pixel with less depth has already been drawn
                if pixel_z > depth_map[idx] {
                    continue;
                }

                depth_map[idx] = pixel_z;
                frame_buf[idx] = Some(color);
            }
        }
    }
}

// Diffs current frame against prev and only emits terminal updates for changed cells
// that way no full Clear every frame
pub fn flush_frame(
    stdout: &mut std::io::Stdout,
    current: &[Option<Color>],
    prev: &mut [Option<Color>],
    width: u16,
    height: u16,
) -> std::io::Result<()> {
    for y in 0..height {
        for x in 0..width {
            let idx = y as usize * width as usize + x as usize;
            if current[idx] != prev[idx] {
                match current[idx] {
                    Some(color) => queue!(
                        stdout,
                        cursor::MoveTo(x, y),
                        style::PrintStyledContent("█".with(color))
                    )?,
                    None => queue!(
                        stdout,
                        cursor::MoveTo(x, y),
                        style::Print(" ")
                    )?,
                }
            }
        }
    }
    prev.copy_from_slice(current);
    stdout.flush()
}
