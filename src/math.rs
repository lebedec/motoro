use std::ops::{Add, Div, Mul, Neg, Sub};

/// Math module is designed for simple vector and matrix processing.
/// Therefore, almost all of its operators are overloaded to perform standard operations as defined
/// in linear algebra. In cases where an operation is not defined in linear algebra,
/// the operation is typically done component-wise, where the operation is performed on
/// each individual element of the vector or matrix.

pub type Vec2 = [f32; 2];
pub type Vec2u = [u32; 2];
pub type Vec2i = [i32; 2];
pub type Vec2s = [usize; 2];

pub type Vec3 = [f32; 3];

pub type Vec4 = [f32; 4];

pub trait VecGrid {
    fn offset_in(&self, grid: Vec2s) -> usize;
    fn position_of(&self, index: usize) -> Vec2s;
}

impl VecGrid for Vec2s {
    fn offset_in(&self, grid: Vec2s) -> usize {
        self.x() + self.y() * grid.x()
    }

    fn position_of(&self, index: usize) -> Vec2s {
        [index % self.x(), index / self.x()]
    }
}

trait VecResize<T> {
    fn resize<const R: usize>(self) -> [T; R];
}

impl<T, const N: usize> VecResize<T> for [T; N]
where
    T: Copy + Default,
{
    fn resize<const R: usize>(self) -> [T; R] {
        let mut result = [T::default(); R];
        for i in 0..R.min(N) {
            result[i] = self[i];
        }
        result
    }
}

pub trait VecComponents<T> {
    fn x(&self) -> T;
    fn y(&self) -> T;
    fn z(&self) -> T;
    fn w(&self) -> T;
    fn r(&self) -> T;
    fn g(&self) -> T;
    fn b(&self) -> T;
    fn a(&self) -> T;
    fn xy(&self) -> [T; 2];
    fn xyz(&self) -> [T; 3];
    fn rgb(&self) -> [T; 3];
}

impl<T, const N: usize> VecComponents<T> for [T; N]
where
    T: Copy + Default,
{
    #[inline(always)]
    fn x(&self) -> T {
        self[0]
    }

    #[inline(always)]
    fn y(&self) -> T {
        self[1]
    }

    #[inline(always)]
    fn z(&self) -> T {
        self[2]
    }

    #[inline(always)]
    fn w(&self) -> T {
        self[3]
    }

    #[inline(always)]
    fn r(&self) -> T {
        self[0]
    }

    #[inline(always)]
    fn g(&self) -> T {
        self[1]
    }

    #[inline(always)]
    fn b(&self) -> T {
        self[2]
    }

    #[inline(always)]
    fn a(&self) -> T {
        self[3]
    }

    #[inline(always)]
    fn xy(&self) -> [T; 2] {
        [self[0], self[1]]
    }

    #[inline(always)]
    fn xyz(&self) -> [T; 3] {
        [self[0], self[1], self[2]]
    }

    #[inline(always)]
    fn rgb(&self) -> [T; 3] {
        [self[0], self[1], self[2]]
    }
}

pub fn vec2_aabb(points: &[Vec2]) -> (Vec2, Vec2) {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for point in points {
        if point.x() < min_x {
            min_x = point.x();
        }
        if point.x() > max_x {
            max_x = point.x();
        }
        if point.y() < min_y {
            min_y = point.y();
        }
        if point.y() > max_y {
            max_y = point.y();
        }
    }
    ([min_x, min_y], [max_x, max_y])
}

pub trait VecSnap {
    fn snap(&self, tile: Self) -> Self;

    fn grid(&self, tile: Self) -> Self;
}

impl<const N: usize> VecSnap for [f32; N] {
    fn snap(&self, tile: Self) -> Self {
        self.grid(tile).mul(tile)
    }

    fn grid(&self, tile: Self) -> Self {
        self.div(tile).floor()
    }
}

pub trait VecFloor {
    fn floor(&self) -> Self;
    fn round(&self) -> Self;
    fn ceil(&self) -> Self;
}

impl<const N: usize> VecFloor for [f32; N] {
    fn floor(&self) -> Self {
        self.map(|value| value.floor())
    }
    fn round(&self) -> Self {
        self.map(|value| value.round())
    }
    fn ceil(&self) -> Self {
        self.map(|value| value.ceil())
    }
}

pub trait VecNeighbors<T>
where
    Self: Sized,
{
    fn subgrid(&self, radius: usize, grid: Self) -> Vec<Self>;
}

impl VecNeighbors<usize> for Vec2s {
    fn subgrid(&self, radius: usize, grid: Vec2s) -> Vec<Vec2s> {
        let [cx, cy] = *self;
        let min_y = cy - radius.min(cy);
        let max_y = (cy + radius + 1).min(grid.y());
        let min_x = cx - radius.min(cx);
        let max_x = (cx + radius + 1).max(grid.x());
        let mut result = vec![];
        for y in min_y..max_y {
            for x in min_x..max_x {
                result.push([x, y])
            }
        }
        result
    }
}

pub trait VecSpace<T> {
    fn space(&self) -> T;

    fn contains(&self, target: Self) -> bool;
}

impl<T, const N: usize> VecSpace<T> for [T; N]
where
    T: Copy + Default + Mul<Output = T> + PartialOrd,
{
    fn space(&self) -> T {
        let mut result = self[0];
        for i in 1..N {
            result = result * self[i];
        }
        result
    }

    fn contains(&self, target: Self) -> bool {
        for i in 0..N {
            if target[i] < T::default() || target[i] >= self[i] {
                return false;
            }
        }
        true
    }
}

pub trait VecMagnitude<const N: usize>
where
    Self: Sized + Copy,
{
    fn dot(self, other: Self) -> f32;

    fn sqr_magnitude(self) -> f32 {
        self.dot(self)
    }

    fn magnitude(&self) -> f32 {
        self.sqr_magnitude().sqrt()
    }

    fn normal(&self) -> [f32; N];
}

impl<const N: usize> VecMagnitude<N> for [f32; N] {
    fn dot(self, other: Self) -> f32 {
        let mut result = 0.0;
        for i in 0..N {
            result += self[i] * other[i];
        }
        result
    }

    fn normal(&self) -> [f32; N] {
        let magnitude = self.magnitude();
        if magnitude > 0.0 {
            self.div(magnitude)
        } else {
            [0.0; N]
        }
    }
}

impl<const N: usize> VecMagnitude<N> for [u32; N] {
    fn dot(self, other: Self) -> f32 {
        let mut result = 0.0;
        for i in 0..N {
            result += (self[i] * other[i]) as f32;
        }
        result
    }

    fn normal(&self) -> [f32; N] {
        let magnitude = self.magnitude();
        if magnitude > 0.0 {
            self.cast().div(magnitude)
        } else {
            [0.0; N]
        }
    }
}

impl<const N: usize> VecMagnitude<N> for [i32; N] {
    fn dot(self, other: Self) -> f32 {
        let mut result = 0.0;
        for i in 0..N {
            result += (self[i] * other[i]) as f32;
        }
        result
    }

    fn normal(&self) -> [f32; N] {
        let magnitude = self.magnitude();
        if magnitude > 0.0 {
            self.cast().div(magnitude)
        } else {
            [0.0; N]
        }
    }
}

impl<const N: usize> VecMagnitude<N> for [usize; N] {
    fn dot(self, other: Self) -> f32 {
        let mut result = 0.0;
        for i in 0..N {
            result += (self[i] * other[i]) as f32;
        }
        result
    }

    fn normal(&self) -> [f32; N] {
        let magnitude = self.magnitude();
        if magnitude > 0.0 {
            self.cast().div(magnitude)
        } else {
            [0.0; N]
        }
    }
}

pub trait VecNeg {
    fn neg(&self) -> Self;
}

impl<T, const N: usize> VecNeg for [T; N]
where
    T: Copy + Neg<Output = T>,
{
    fn neg(&self) -> Self {
        self.map(|value| value.neg())
    }
}

pub trait VecCast<T, const N: usize> {
    fn cast(&self) -> [T; N];
}

impl<const N: usize> VecCast<u32, N> for [f32; N] {
    fn cast(&self) -> [u32; N] {
        self.map(|value| value as u32)
    }
}

impl<const N: usize> VecCast<usize, N> for [f32; N] {
    fn cast(&self) -> [usize; N] {
        self.map(|value| value as usize)
    }
}

impl<const N: usize> VecCast<isize, N> for [f32; N] {
    fn cast(&self) -> [isize; N] {
        self.map(|value| value as isize)
    }
}

impl<const N: usize> VecCast<f32, N> for [usize; N] {
    fn cast(&self) -> [f32; N] {
        self.map(|value| value as f32)
    }
}

impl<const N: usize> VecCast<isize, N> for [usize; N] {
    fn cast(&self) -> [isize; N] {
        self.map(|value| value as isize)
    }
}

impl<const N: usize> VecCast<f32, N> for [isize; N] {
    fn cast(&self) -> [f32; N] {
        self.map(|value| value as f32)
    }
}

impl<const N: usize> VecCast<usize, N> for [isize; N] {
    fn cast(&self) -> [usize; N] {
        self.map(|value| value as usize)
    }
}

impl<const N: usize> VecCast<f32, N> for [u32; N] {
    fn cast(&self) -> [f32; N] {
        self.map(|value| value as f32)
    }
}

impl<const N: usize> VecCast<f32, N> for [i32; N] {
    fn cast(&self) -> [f32; N] {
        self.map(|value| value as f32)
    }
}

pub trait VecArith<C> {
    fn add(&self, other: C) -> Self;
    fn sub(&self, other: C) -> Self;
    fn mul(&self, other: C) -> Self;
    fn div(&self, other: C) -> Self;
}

impl<T, const N: usize> VecArith<[T; N]> for [T; N]
where
    T: Copy + Default + Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T>,
{
    fn add(&self, other: [T; N]) -> Self {
        let mut result = [T::default(); N];
        for i in 0..N {
            result[i] = self[i] + other[i];
        }
        result
    }

    fn sub(&self, other: [T; N]) -> Self {
        let mut result = [T::default(); N];
        for i in 0..N {
            result[i] = self[i] - other[i];
        }
        result
    }

    fn mul(&self, other: [T; N]) -> Self {
        let mut result = [T::default(); N];
        for i in 0..N {
            result[i] = self[i] * other[i];
        }
        result
    }

    fn div(&self, other: [T; N]) -> Self {
        let mut result = [T::default(); N];
        for i in 0..N {
            result[i] = self[i] / other[i];
        }
        result
    }
}

impl<T, const N: usize> VecArith<T> for [T; N]
where
    T: Copy + Mul<Output = T> + Div<Output = T> + Add<Output = T> + Sub<Output = T>,
{
    fn add(&self, other: T) -> Self {
        self.map(|value| value + other)
    }

    fn sub(&self, other: T) -> Self {
        self.map(|value| value - other)
    }

    fn mul(&self, other: T) -> Self {
        self.map(|value| value * other)
    }

    fn div(&self, other: T) -> Self {
        self.map(|value| value / other)
    }
}

pub fn vec3_cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
        mat4_row(a, 0).dot(col),
        mat4_row(a, 1).dot(col),
        mat4_row(a, 2).dot(col),
        mat4_row(a, 3).dot(col),
    ]
}

/// based on [Microsoft Matrix.LookAtRH](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bb281711(v=vs.85))
pub fn mat4_look_at_rh(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let z = eye.sub(target).normal();
    let x = vec3_cross(up, z).normal();
    let y = vec3_cross(z, x);
    [
        [x[0], y[0], z[0], 0.0],
        [x[1], y[1], z[1], 0.0],
        [x[2], y[2], z[2], 0.0],
        [-x.dot(eye), -y.dot(eye), -z.dot(eye), 1.0],
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
