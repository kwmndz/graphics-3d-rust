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

    *m = Matrix([
        [a_sc.1 * b_sc.1, a_sc.0 * b_sc.1, -b_sc.0],
        [a_sc.1 * b_sc.0 * c_sc.0 - a_sc.0 * c_sc.1, 
            a_sc.0 * b_sc.0 * c_sc.0 + a_sc.1 * c_sc.1, 
            b_sc.1 * c_sc.0],
        [a_sc.1 * b_sc.0 * c_sc.1 + a_sc.0 * c_sc.0, 
            a_sc.0 * b_sc.0 * c_sc.1 - a_sc.1 * c_sc.0, 
            b_sc.1 * c_sc.1]
    ]);
}

const CORNERS: [Vector; 8] = [
    Vector([1.0, 1.0, 1.0]),
    Vector([-1.0, 1.0, 1.0]),
    Vector([1.0, -1.0, 1.0]),
    Vector([-1.0, -1.0, 1.0]),
    Vector([1.0, 1.0, -1.0]),
    Vector([-1.0, 1.0, -1.0]),
    Vector([1.0, -1.0, -1.0]),
    Vector([-1.0, -1.0, -1.0])
];

fn main() {
    println!("Hello, world!");
}
