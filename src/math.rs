pub type Vec2 = [f32; 2];

pub fn vec2_add(a: Vec2, b: Vec2) -> Vec2 {
    [a[0] + b[0], a[1] + b[1]]
}

pub fn vec2_scale(a: Vec2, s: f32) -> Vec2 {
    [a[0] * s, a[1] * s]
}

pub type Vec3 = [f32; 3];

pub fn vec3_neg(a: Vec3) -> Vec3 {
    [-a[0], -a[1], -a[2]]
}

pub fn vec3_scale(a: Vec3, s: f32) -> Vec3 {
    [a[0] * s, a[1] * s, a[2] * s]
}

pub fn vec3_magnitude(a: Vec3) -> f32 {
    a[0] * a[0] + a[1] * a[1] + a[2] * a[2]
}

pub fn vec3_normalized(a: Vec3) -> Vec3 {
    vec3_scale(a, 1.0 / vec3_magnitude(a))
}

pub fn vec3_cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

pub fn vec3_sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn vec3_dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub type Vec4 = [f32; 4];

pub fn vec4_mul(a: Vec4, b: Vec4) -> Vec4 {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

pub fn vec4_dot(a: Vec4, b: Vec4) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3]
}

/// A statically sized column-major 4x4 matrix.
pub type Mat4 = [[f32; 4]; 4];

#[inline]
pub fn mat4_identity() -> Mat4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn mat4_from_scale(scale: Vec3) -> Mat4 {
    [
        [scale[0], 0.0, 0.0, 0.0],
        [0.0, scale[1], 0.0, 0.0],
        [0.0, 0.0, scale[2], 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub fn mat4_from_translation(delta: Vec3) -> Mat4 {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [delta[0], delta[1], delta[2], 1.0],
    ]
}

pub fn mat4_row(matrix: Mat4, row: usize) -> Vec4 {
    [
        matrix[0][row],
        matrix[1][row],
        matrix[2][row],
        matrix[3][row],
    ]
}

pub fn mat4_mul(a: Mat4, b: Mat4) -> Mat4 {
    [
        mat4_mul_col(a, b[0]),
        mat4_mul_col(a, b[1]),
        mat4_mul_col(a, b[2]),
        mat4_mul_col(a, b[3]),
    ]
}

pub fn mat4_mul_col(a: Mat4, col: Vec4) -> Vec4 {
    [
        vec4_dot(mat4_row(a, 0), col),
        vec4_dot(mat4_row(a, 1), col),
        vec4_dot(mat4_row(a, 2), col),
        vec4_dot(mat4_row(a, 3), col),
    ]
}

/// based on [Microsoft Matrix.LookAtRH](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bb281711(v=vs.85))
pub fn mat4_look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let z = vec3_normalized(vec3_sub(eye, target));
    let x = vec3_normalized(vec3_cross(up, z));
    let y = vec3_cross(z, x);
    [
        [x[0], y[0], z[0], 0.0],
        [x[1], y[1], z[1], 0.0],
        [x[2], y[2], z[2], 0.0],
        [-vec3_dot(x, eye), -vec3_dot(y, eye), -vec3_dot(z, eye), 1.0],
    ]
}

pub fn mat4_orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    z_near: f32,
    z_far: f32,
) -> Mat4 {
    let mut matrix = mat4_identity();
    set_left_and_right(&mut matrix, left, right);
    set_bottom_and_top(&mut matrix, bottom, top);
    set_z_near_and_z_far(&mut matrix, z_near, z_far);
    matrix
}

/// Sets the view cuboid offsets along the X axis.
#[inline]
fn set_left_and_right(matrix: &mut Mat4, left: f32, right: f32) {
    matrix[0][0] = 2.0 / (right - left);
    matrix[3][0] = -(right + left) / (right - left);
}

/// Sets the view cuboid offsets along the Y axis.
#[inline]
pub fn set_bottom_and_top(matrix: &mut Mat4, bottom: f32, top: f32) {
    matrix[1][1] = 2.0 / (top - bottom);
    matrix[3][1] = -(top + bottom) / (top - bottom);
}

/// Sets the near and far plane offsets of the view cuboid.
#[inline]
pub fn set_z_near_and_z_far(matrix: &mut Mat4, z_near: f32, z_far: f32) {
    matrix[2][2] = -2.0 / (z_far - z_near);
    matrix[3][2] = -(z_far + z_near) / (z_far - z_near);
}

#[inline]
pub fn mat4_prepend_scale(matrix: &mut Mat4, scale: Vec3) {
    matrix[1][0] *= scale[0];
    matrix[1][1] *= scale[1];
    matrix[1][2] *= scale[2];
}
