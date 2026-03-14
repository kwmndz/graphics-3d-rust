#[derive(Clone, Copy, Debug)]
pub struct Vector(pub [f32; 3]);

#[derive(Clone, Copy, Debug)]
pub struct Matrix(pub [[f32; 3]; 3]);

// mult 3x3 matrix with 3x1 vector
pub fn matrix_mult(m: &Matrix, v: &Vector) -> Vector {
    let [mx, my, mz] = &m.0;
    let [x, y, z] = v.0;

    Vector([
        x*mx[0] + y*my[0] + z*mz[0],
        x*mx[1] + y*my[1] + z*mz[1],
        x*mx[2] + y*my[2] + z*mz[2]
    ])
}

// angles a, b, c represent rotation about z, y, x (yaw, pitch, roll) in radians
pub fn rotation_matrix(m: &mut Matrix, a: f32, b: f32, c: f32) {
    let a_sc = a.sin_cos();
    let b_sc = b.sin_cos();
    let c_sc = c.sin_cos();

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
