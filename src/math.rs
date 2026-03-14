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

// matrix * matrix 
pub fn matrix_matrix_mult(a: &Matrix, b: &Matrix) -> Matrix {
    let mut result = [[0.0f32; 3]; 3];
    for col in 0..3 {
        for row in 0..3 {
            for k in 0..3 {
                result[col][row] += a.0[k][row] * b.0[col][k];
            }
        }
    }
    Matrix(result)
}

// single-axis rotation matrices (column-major storage)
pub fn rotation_x(angle: f32) -> Matrix {
    let (s, c) = angle.sin_cos();
    Matrix([
        [1.0, 0.0, 0.0],
        [0.0, c,   s  ],
        [0.0, -s,  c  ],
    ])
}

pub fn rotation_y(angle: f32) -> Matrix {
    let (s, c) = angle.sin_cos();
    Matrix([
        [c,   0.0, -s ],
        [0.0, 1.0, 0.0],
        [s,   0.0, c  ],
    ])
}

// Gram-Schmidt orthonormalization, to fix float drift that accumulates
// when multiplying many small rotation matrices together over time
pub fn orthonormalize(m: &mut Matrix) {
    let dot = |a: [f32; 3], b: [f32; 3]| a[0]*b[0] + a[1]*b[1] + a[2]*b[2];
    let normalize = |v: [f32; 3]| -> [f32; 3] {
        let len = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
        if len == 0.0 { return v; }
        [v[0]/len, v[1]/len, v[2]/len]
    };

    let u0 = normalize(m.0[0]);

    let col1 = m.0[1];
    let d = dot(col1, u0);
    let u1 = normalize([col1[0] - d*u0[0], col1[1] - d*u0[1], col1[2] - d*u0[2]]);

    let col2 = m.0[2];
    let d0 = dot(col2, u0);
    let d1 = dot(col2, u1);
    let u2 = normalize([
        col2[0] - d0*u0[0] - d1*u1[0],
        col2[1] - d0*u0[1] - d1*u1[1],
        col2[2] - d0*u0[2] - d1*u1[2],
    ]);

    m.0[0] = u0;
    m.0[1] = u1;
    m.0[2] = u2;
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
