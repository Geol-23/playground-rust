use std::cmp::{ PartialOrd, PartialEq };
use std::default::Default;
use std::fmt::Debug;
use std::iter::Sum;
use std::ops::{Add, Mul, Neg, Sub};


macro_rules! mark_type {
    ($trait:ident, $($t:ty),+ $(,)?) => {
        $(
            impl $trait for $t {}
        )+
    };
}

pub trait Scalar:
    Debug +
    Copy +
    Default +
    PartialOrd +
    PartialEq +
    Add<Output = Self> +
    Sub<Output = Self> +
    Mul<Output = Self> +
    Sum<Self>
{}

pub trait Natural: Scalar {}

pub trait Integer:
    Scalar +
    Neg<Output = Self>
{}

pub trait Real:
    Scalar +
    Neg<Output = Self>
{}

mark_type!(Scalar, u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);
mark_type!(Natural, u8, u16, u32, u64, usize);
mark_type!(Integer, i8, i16, i32, i64, isize);
mark_type!(Real, f32, f64);
