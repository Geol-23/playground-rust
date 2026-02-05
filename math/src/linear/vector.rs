use std::cmp::PartialEq;
use std::convert::From;
use std::default::Default;
use std::ops::{ Add, Neg, Sub, Mul, Index, IndexMut };

use crate::traits::Scalar;
use crate::linear::traits::Lerp;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Vector<T: Scalar, const N: usize> {
    v: [T; N]
}

impl<T: Scalar, const N: usize> Vector<T, N> {
    pub fn from_arr(v: [T; N]) -> Self {
        Self { v }
    }

    pub fn splat(value: T) -> Self {
        Self { v: [value; N] }
    }

    pub fn from_fn<F>(f: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        Self {
            v: std::array::from_fn(f),
        }
    }

    pub fn dot(self: &Self, other: &Self) -> T {
        self.v
            .into_iter()
            .zip(other.v.into_iter())
            .map(|(a, b)| a * b)
            .sum()
    }

    pub fn abs2(self: &Self) -> T {
        self.v
            .into_iter()
            .map(|x| x * x)
            .sum()
    }
}


impl<T: Scalar + Into<f64>, const N: usize> Vector<T, N> {
    pub fn abs(&self) -> f64 {
        let sum_sq: f64 = self.v.iter().map(|&x| {
            let xf: f64 = x.into();
            xf * xf
        }).sum();
        sum_sq.sqrt()
    }

    pub fn to_f64(self: Self) -> Vector<f64, N> {
        Vector {
            v: self.v.map(|x| x.into())
        }
    }
}

impl<T: Scalar, const N: usize> Default for Vector<T, N> {
    fn default() -> Self {
        Self {
            v: std::array::from_fn(|_| T::default())
        }
    }
}


impl<T: Scalar, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(value: [T; N]) -> Self {
        Self::from_arr(value)
    }
}


// impl<T, U, const N: usize> From<Vector<T, N>> for Vector<U, N>
// where
//     T: Into<U> + Scalar,
//     U: Scalar
// {
//     fn from(other: Vector<T, N>) -> Self {
//         Vector {
//             v: other.v.map(|x| x.into()),
//         }
//     }
// }


impl<T: Scalar, const N: usize> Index<usize> for Vector<T, N> {
    type Output = T;

    fn index(self: &Self, index: usize) -> &Self::Output {
        &self.v[index]
    }
}


impl<T: Scalar, const N: usize> IndexMut<usize> for Vector<T, N> {
    fn index_mut(self: &mut Self, index: usize) -> &mut Self::Output {
        &mut self.v[index]
    }
}


impl<T: Scalar, const N: usize> Add for Vector<T, N> {
    type Output = Self;

    fn add(self: Self, other: Self) -> Self {
        let mut out: Vector<T, N> = Self::default();
        for i in 0..N { out[i] = self[i] + other[i] }
        out
    }
}


impl<T: Scalar + Neg<Output = T>, const N: usize> Neg for Vector<T, N> {
    type Output = Self;

    fn neg(self: Self) -> Self {
        let mut out: Vector<T, N> = Self::default();
        for i in 0..N { out[i] = -self[i] }
        out
    }
}


impl<T: Scalar, const N: usize> Sub for Vector<T, N> {
    type Output = Self;

    fn sub(self: Self, other: Self) -> Self {
        let mut out: Vector<T, N> = Self::default();
        for i in 0..N { out[i] = self[i] - other[i] }
        out
    }
}

impl<T: Scalar, const N: usize> Mul<T> for Vector<T, N> {
    type Output = Self;

    fn mul(self, other: T) -> Self {
        let mut out = Self::default();
        for i in 0..N {
            out[i] = self[i] * other;
        }
        out
    }
}


impl<T: Scalar, const N: usize> Vector<T, N> {
    pub fn map_into<U: Scalar + From<T>>(self) -> Vector<U, N> {
        Vector { v: self.v.map(|x| x.into()) }
    }
}



impl<T: Scalar, const N: usize> Lerp for Vector<T, N>
where
    f64: From<T>
{
    type Output = Vector<f64, N>;

   fn lerp(self: Self, other: Self, t: f64) -> Vector<f64, N> {
       let self_f64 = self.map_into();
       self_f64 + (other.map_into() - self_f64) * t
   }
}


// Vector 2D specialization
pub type Vector2D<T> = Vector<T, 2>;
pub type Vec2<T> = Vector2D<T>;

impl<T: Scalar> Vec2<T> {
    pub fn x(self: Self) -> T { self.v[0] }
    pub fn y(self: Self) -> T { self.v[1] }

    pub fn new(x: T, y: T) -> Self {
        Self { v: [x, y] }
    }
}


// Vector 3D specialisation
pub type Vector3D<T> = Vector<T, 3>;
pub type Vec3<T> = Vector3D<T>;

impl<T: Scalar> Vec3<T> {
    pub fn x(self: Self) -> T { self.v[0] }
    pub fn y(self: Self) -> T { self.v[1] }
    pub fn z(self: Self) -> T { self.v[2] }

    pub fn new(x: T, y: T, z: T) -> Self {
        Self { v: [x, y, z] }
    }
}
