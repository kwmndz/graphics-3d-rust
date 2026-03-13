use core::f32;
use std::io::{stdout, Write};
use std::thread;
use std::time::Duration;
use std::collections::HashMap;

use crossterm::{
    execute,
    style::{self, Stylize, Color},
    terminal::{Clear,ClearType, size},
    cursor, queue
};


#[derive(Clone, Copy, Debug)]
struct Vector([f32; 3]);

#[derive(Clone, Copy, Debug)]
struct Matrix([[f32; 3]; 3]);

fn matrix_mult(m: &Matrix, v: &Vector) -> Vector {
    let [mx, my, mz] = &m.0;
    let [x, y, z] = v.0;

    Vector([
        x*mx[0] + y*my[0] + z*mz[0],
        x*mx[1] + y*my[1] + z*mz[1],
        x*mx[2] + y*my[2] + z*mz[2]
    ])
}

// angles a, b, c represent roation about z,y,x
// yaw, pitch, roll
fn rotation_matrix(m: &mut Matrix, a: f32, b:f32, c:f32) {
    let a_sc = a.to_radians().sin_cos();
    let b_sc = b.to_radians().sin_cos();
    let c_sc = c.to_radians().sin_cos();

    m.0[0][0] = a_sc.1 * b_sc.1;
    m.0[0][1] = a_sc.0 * b_sc.1;
    m.0[0][2] = -b_sc.0;

    m.0[1][0] = a_sc.1 * b_sc.0 * c_sc.0 - a_sc.0 * c_sc.1;
    m.0[1][1] = a_sc.0 * b_sc.0 * c_sc.0 + a_sc.1 * c_sc.1;
    m.0[1][2] = b_sc.1 * c_sc.0;

    m.0[2][0] = a_sc.1 * b_sc.0 * c_sc.1 + a_sc.0 * c_sc.0;
    m.0[2][1] = a_sc.0 * b_sc.0 * c_sc.1 - a_sc.1 * c_sc.0;
    m.0[2][2] = b_sc.1 * c_sc.1;
}

fn convert_to_screen(v: &Vector, w: f32, h: f32) -> Option<(u16, u16, f32)> {
    let [x, y, z] = v.0;
    let scale = 20.0;
    let ooz = 1.0 / z;

    let sx = x * ooz * scale + w * 0.5;
    let sy = h * 0.5 - y * ooz * scale;

    if sx >= 0.0 && sx < w && sy >= 0.0 && sy < h {
        Some((sx as u16, sy as u16, z))
    } else {
        None
    }
}

fn draw_line(
    stdout: &mut std::io::Stdout,
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
    a:  usize,
    b:  usize
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
            // style::PrintStyledContent("█".magenta())
            style::PrintStyledContent("@".magenta())
        )?;
    }

    Ok(())
}

fn sign(p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) -> f32 {
    (p1.0 - p3.0) * (p2.1 - p3.1) - (p2.0 - p3.0) * (p1.1 - p3.1)
}

fn in_triangle(pt: (f32, f32), v1: (f32, f32), v2: (f32, f32), v3: (f32, f32)) -> bool {
    let d1: f32 = sign(pt, v1, v2);
    let d2: f32 = sign(pt, v2, v3);
    let d3: f32 = sign(pt, v3, v1);

    let has_neg: bool = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
    let has_pos: bool = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);

    !(has_neg && has_pos)

}

fn draw_faces(
    stdout: &mut std::io::Stdout, projected: &[Option<(u16, u16, f32)>; 8], 
    c: (usize, usize, usize), color: Color, depthMap: &mut HashMap<(u16, u16), f32>
) -> std::io::Result<()> {

    if let (Some((x0, y0, z0)), Some((x1, y1, z1)), Some((x2, y2, z2))) =
        (projected[c.0], projected[c.1], projected[c.2])
    {
        let x0 = x0 as f32;
        let y0 = y0 as f32;

        let x1 = x1 as f32;
        let y1 = y1 as f32;

        let x2 = x2 as f32;
        let y2 = y2 as f32;

        let minx = x2.min(x0.min(x1));
        let miny = y2.min(y0.min(y1));
        let maxx = x2.max(x0.max(x1));
        let maxy = y2.max(y0.max(y1));

        let maxz = z2.max(z0.max(z1));
        // let minz = z2.max(z0.max(z1));

        let dx = maxx - minx;
        let dy = maxy - miny;
        // let dz = maxz - minz;

        let steps = dx.abs() as usize;
        let steps1 = dy.abs() as usize;
        // let steps2: f32 = dz.abs();

        for i in 0..=steps {
            for i1 in 0..=steps1 {
                let t = i as f32 / steps as f32;
                let t1 = i1 as f32 / steps1 as f32;
                let x = minx + dx * t;
                let y = miny + dy * t1;

                if !(in_triangle((x,y), (x0,y0), (x1,y1), (x2,y2))) {
                    continue;
                }

                // dont draw if a pixel with less depth has already beeen drawn
                // sets to infinity if (x,y) isnt a valid entry
                let d = depthMap.entry((x.round() as u16,y.round() as u16)).or_insert(f32::INFINITY);
                if maxz > *d {
                    continue;
                }

                *d = maxz;

                queue!(
                    stdout,
                    cursor::MoveTo(x.round() as u16, y.round() as u16),
                    // style::PrintStyledContent("█".with(color))
                    style::PrintStyledContent("@".with(color))
                )?;
            }
        }

    }
    Ok(())
} 

const CORNERS: [Vector; 8] = [
    Vector([1.0, 1.0, 1.0]), // 0
    Vector([-1.0, 1.0, 1.0]), // 1
    Vector([1.0, -1.0, 1.0]), // 2
    Vector([-1.0, -1.0, 1.0]), // 3 ---
    Vector([1.0, 1.0, -1.0]), // 4
    Vector([-1.0, 1.0, -1.0]), // 5
    Vector([1.0, -1.0, -1.0]), // 6
    Vector([-1.0, -1.0, -1.0]) // 7 ---
];

const EDGES: [(usize, usize); 12] = [
    (0, 1), (1, 3), (3, 2), (2, 0), // front face
    (4, 5), (5, 7), (7, 6), (6, 4), // back face
    (0, 4), (1, 5), (2, 6), (3, 7) // connecting edges
];

fn main() -> std::io::Result<()> {

    // let width: u16;
    // let height: u16;
    // let mut buffer: Vec<Vec<u8>> = vec![vec![b'+';width.into()]; height.into()];
    
    let mut depthMap: HashMap<(u16, u16), f32> = HashMap::new();
    let mut stdout = stdout();
    let (width, height) = size()?;
    thread::sleep(Duration::from_millis(1000));
    execute!(stdout, Clear(ClearType::All), cursor::Hide)?;

    let mut projected: [Option<(u16, u16, f32)>; 8] = [None; 8];

    let mut rot = Matrix([[0.0; 3]; 3]);

    const A: f32 = 25.0;
    const B: f32 = 75.0;
    const C: f32 = 0.0;

    rotation_matrix(&mut rot, A, B, C);

    const FPS: u64 = 60;

    let mut it: f32 = 0.0;

    loop {
        execute!(stdout, Clear(ClearType::All))?;
        for (i, v) in CORNERS.iter().enumerate() {
            rotation_matrix(&mut rot, A+it, B+(it/2.0), C);
            let rotated = matrix_mult(&rot, v);
            let Vector([x, y, z]) = rotated;

            let shifted = Vector([x, y, z + 4.0]);
            projected[i] = convert_to_screen(&shifted, width as f32, height as f32);
        }

        for &(a, b) in &EDGES {
            if let (Some((x0, y0, _z0)), Some((x1, y1, _z1))) = (projected[a], projected[b]) {
                draw_line(&mut stdout, x0, y0, x1, y1, a ,b)?;
            }
        }

        //front face
        draw_faces(&mut stdout, &projected, (0,1,2), Color::Red, &mut depthMap)?;
        draw_faces(&mut stdout, &projected, (1,2,3), Color::Blue, &mut depthMap)?;
        //back face
        draw_faces(&mut stdout, &projected, (4,5,6), Color::Green, &mut depthMap)?;
        draw_faces(&mut stdout, &projected, (5,6,7), Color::DarkGreen, &mut depthMap)?;


        it += 1.0;
        stdout.flush()?;
        depthMap.clear(); // reset depth map for new draw
        thread::sleep(Duration::from_millis(1000 / FPS));
    }
}
