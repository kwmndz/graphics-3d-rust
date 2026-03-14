use std::io::stdout;
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashSet;
use std::panic;

use crossterm::{
    cursor, execute,
    terminal::{self, Clear, ClearType, supports_keyboard_enhancement},
    event::{poll, read, Event, KeyCode,
        KeyEventKind,
        KeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags,
    },
};
use crossterm::style::Color;

mod math;
mod renderer;

use math::{Vector, Matrix, matrix_mult, matrix_matrix_mult, rotation_x, rotation_y, orthonormalize};
use renderer::{convert_to_screen, draw_face, flush_frame};

const CORNERS: [Vector; 8] = [
    Vector([1.0, 1.0, -1.0]),   // 0
    Vector([-1.0, 1.0, -1.0]),  // 1
    Vector([1.0, -1.0, -1.0]),  // 2
    Vector([-1.0, -1.0, -1.0]), // 3
    Vector([1.0, 1.0, 1.0]),    // 4
    Vector([-1.0, 1.0, 1.0]),   // 5
    Vector([1.0, -1.0, 1.0]),   // 6
    Vector([-1.0, -1.0, 1.0])   // 7
];

const EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 3), (3, 2), (2, 0), // front face
    (4, 5), (5, 7), (7, 6), (6, 4), // back face
    (0, 4), (1, 5), (2, 6), (3, 7)  // connecting edges
];

fn draw_faces(
    frame_buf: &mut [Option<Color>],
    projected: &[Option<(u16, u16, f32)>; 8],
    depth_map: &mut Vec<f32>,
    width: u16
) {
    draw_face(frame_buf, projected, (0,1,3), Color::Red,         depth_map, width);
    draw_face(frame_buf, projected, (0,3,2), Color::DarkRed,     depth_map, width);
    draw_face(frame_buf, projected, (5,4,6), Color::Green,       depth_map, width);
    draw_face(frame_buf, projected, (5,6,7), Color::DarkGreen,   depth_map, width);
    draw_face(frame_buf, projected, (0,4,5), Color::DarkBlue,    depth_map, width);
    draw_face(frame_buf, projected, (0,5,1), Color::Blue,        depth_map, width);
    draw_face(frame_buf, projected, (2,3,7), Color::Cyan,        depth_map, width);
    draw_face(frame_buf, projected, (2,7,6), Color::DarkCyan,    depth_map, width);
    draw_face(frame_buf, projected, (4,0,2), Color::Magenta,     depth_map, width);
    draw_face(frame_buf, projected, (4,2,6), Color::DarkMagenta, depth_map, width);
    draw_face(frame_buf, projected, (1,5,7), Color::Yellow,      depth_map, width);
    draw_face(frame_buf, projected, (1,7,3), Color::DarkYellow,  depth_map, width);
}

fn main() -> std::io::Result<()> {
    // restore terminal on crash
    panic::set_hook(Box::new(|info| {
        let _ = terminal::disable_raw_mode();
        eprintln!("{info}");
    }));

    let mut stdout = stdout();
    let (width, height) = terminal::size()?;
    terminal::enable_raw_mode()?;

    let buf_size = width as usize * height as usize;
    let mut depth_map: Vec<f32> = vec![f32::INFINITY; buf_size];
    let mut frame_buf: Vec<Option<Color>> = vec![None; buf_size];
    let mut prev_buf: Vec<Option<Color>> = vec![None; buf_size];

    // basically check if the terminal supports "release" key events
    let enhanced = supports_keyboard_enhancement().unwrap_or(false);

    thread::sleep(Duration::from_millis(1000));
    if enhanced {
        execute!(stdout, Clear(ClearType::All), cursor::Hide,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::REPORT_EVENT_TYPES,
            )
        )?;
    } else {
        execute!(stdout, Clear(ClearType::All), cursor::Hide)?;
    }

    let mut projected: [Option<(u16, u16, f32)>; 8] = [None; 8];

    // identity matrix
    let mut rot = Matrix([
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ]);

    const IT_STEP: f32 = 1.5 * std::f32::consts::PI / 180.0;
    const FPS: u64 = 60;
    const ZSHIFT: f32 = 3.2;

    let frame_duration = Duration::from_millis(1000 / FPS);
    let mut held_keys: HashSet<KeyCode> = HashSet::new();
    let mut frame_count: u32 = 0;

    'main: loop {
        let frame_start = Instant::now();

        frame_buf.fill(None);
        depth_map.fill(f32::INFINITY);

        if enhanced {
            // loop all events and track held keys via press/release pairs
            // only works if terminal emulator supports it
            while poll(Duration::ZERO)? {
                if let Event::Key(ke) = read()? {
                    match ke.kind {
                        KeyEventKind::Press   => { held_keys.insert(ke.code); }
                        KeyEventKind::Release => { held_keys.remove(&ke.code); }
                        _ => {}
                    }
                }
            }

            if held_keys.contains(&KeyCode::Esc) { break 'main; }

            if held_keys.contains(&KeyCode::Left)  { rot = matrix_matrix_mult(&rotation_y(-IT_STEP), &rot); }
            if held_keys.contains(&KeyCode::Right) { rot = matrix_matrix_mult(&rotation_y( IT_STEP), &rot); }
            if held_keys.contains(&KeyCode::Up)    { rot = matrix_matrix_mult(&rotation_x(-IT_STEP), &rot); }
            if held_keys.contains(&KeyCode::Down)  { rot = matrix_matrix_mult(&rotation_x( IT_STEP), &rot); }
        } else {
            // fallback, one axis at a time, one step per press/repeat event
            while poll(Duration::ZERO)? {
                if let Event::Key(ke) = read()? && ke.kind == KeyEventKind::Press{
                    match ke.code {
                        KeyCode::Esc   => break 'main,
                        KeyCode::Left  => { rot = matrix_matrix_mult(&rotation_y(-IT_STEP), &rot); }
                        KeyCode::Right => { rot = matrix_matrix_mult(&rotation_y( IT_STEP), &rot); }
                        KeyCode::Up    => { rot = matrix_matrix_mult(&rotation_x(-IT_STEP), &rot); }
                        KeyCode::Down  => { rot = matrix_matrix_mult(&rotation_x( IT_STEP), &rot); }
                        _ => {}
                    }
                }
            }
        }

        // re orthonormalize every 100 frames to correct float drift
        frame_count += 1;
        if frame_count.is_multiple_of(100) {
            orthonormalize(&mut rot);
        }

        for (i, v) in CORNERS.iter().enumerate() {
            let rotated = matrix_mult(&rot, v);
            let Vector([x, y, z]) = rotated;
            let shifted = Vector([x, y, z + ZSHIFT]);
            projected[i] = convert_to_screen(&shifted, width as f32, height as f32);
        }

        draw_faces(&mut frame_buf, &projected, &mut depth_map, width);
        flush_frame(&mut stdout, &frame_buf, &mut prev_buf, width, height)?;

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            thread::sleep(frame_duration - elapsed);
        }
    }

    execute!(stdout, Clear(ClearType::All), PopKeyboardEnhancementFlags)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
