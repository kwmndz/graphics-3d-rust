use std::io::stdout;
use std::thread;
use std::time::Duration;

use crossterm::{
    execute,
    terminal::{Clear, ClearType, size},
    cursor,
};
use crossterm::style::Color;

mod math;
mod renderer;

use math::{Vector, Matrix, matrix_mult, rotation_matrix};
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

// to cleanup the main loop
fn draw_faces(
    frame_buf: &mut [Option<Color>],
    projected: &[Option<(u16, u16, f32)>; 8],
    depth_map: &mut Vec<f32>,
    width: u16
) {
    // front face (z=-1, outward normal = -z)
    draw_face(frame_buf, projected, (0,1,3), Color::Red, depth_map, width);
    draw_face(frame_buf, projected, (0,3,2), Color::DarkRed, depth_map, width);
    // back face (z=+1, outward normal = +z)
    draw_face(frame_buf, projected, (5,4,6), Color::Green, depth_map, width);
    draw_face(frame_buf, projected, (5,6,7), Color::DarkGreen, depth_map, width);

    // top face (y=+1, outward normal = +y)
    draw_face(frame_buf, projected, (0,4,5), Color::DarkBlue, depth_map, width);
    draw_face(frame_buf, projected, (0,5,1), Color::Blue, depth_map, width);

    // bottom face (y=-1, outward normal = -y)
    draw_face(frame_buf, projected, (2,3,7), Color::Cyan, depth_map, width);
    draw_face(frame_buf, projected, (2,7,6), Color::DarkCyan, depth_map, width);

    // right face (x=+1, outward normal = +x)
    draw_face(frame_buf, projected, (4,0,2), Color::Magenta, depth_map, width);
    draw_face(frame_buf, projected, (4,2,6), Color::DarkMagenta, depth_map, width);

    // left face (x=-1, outward normal = -x)
    draw_face(frame_buf, projected, (1,5,7), Color::Yellow, depth_map, width);
    draw_face(frame_buf, projected, (1,7,3), Color::DarkYellow, depth_map, width);
}

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    // width and height of terminal
    let (width, height) = size()?;

    // use flat vector for buffer and depth_map
    // depth_map basically works as a hash map in this case
    let buf_size = width as usize * height as usize;
    // preset to infinity b/c depth inc as you get further from camera
    let mut depth_map: Vec<f32> = vec![f32::INFINITY; buf_size];

    // off screen frame buffers
    // used in flush_frame to only make terminal I/O updates where changes happened
    let mut frame_buf: Vec<Option<Color>> = vec![None; buf_size];
    let mut prev_buf: Vec<Option<Color>> = vec![None; buf_size];

    thread::sleep(Duration::from_millis(1000));
    execute!(stdout, Clear(ClearType::All), cursor::Hide)?;

    // points projected to screen, with orig Z-value for z-buffer
    let mut projected: [Option<(u16, u16, f32)>; 8] = [None; 8];
    let mut rot = Matrix([[0.0; 3]; 3]);

    // convert to radians b/c rotation matrix needs radians
    const A: f32 = 25.0 * std::f32::consts::PI / 180.0;
    const B: f32 = 75.0 * std::f32::consts::PI / 180.0;
    const C: f32 = 25.0 * std::f32::consts::PI / 180.0;
    // arbitrary iterator so the cube spins
    const IT_STEP: f32 = 1.2 * std::f32::consts::PI / 180.0;

    rotation_matrix(&mut rot, A, B, C);

    const FPS: u64 = 60;
    const ZSHIFT: f32 = 3.2; // shift cube away from observer
    let mut it: f32 = 0.0;

    loop {
        // reset per-frame buffers
        frame_buf.fill(None);
        depth_map.fill(f32::INFINITY);

        rotation_matrix(&mut rot, A + it, B + it * 0.73, C + it * 1.31);
        for (i, v) in CORNERS.iter().enumerate() {
            let rotated = matrix_mult(&rot, v);
            let Vector([x, y, z]) = rotated;

            let shifted = Vector([x, y, z + ZSHIFT]);
            projected[i] = convert_to_screen(&shifted, width as f32, height as f32);
        }

        // draw all faces of the square to buffer
        draw_faces(&mut frame_buf, &projected, &mut depth_map, width);

        it += IT_STEP;
        flush_frame(&mut stdout, &frame_buf, &mut prev_buf, width, height)?;
        thread::sleep(Duration::from_millis(1000 / FPS));
    }
}
