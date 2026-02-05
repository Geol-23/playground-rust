use std::cmp::PartialEq;
use std::convert::From;
use std::ops::Mul;

use crate::linear::Vector2D;
use crate::traits::Scalar;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rect<T: Scalar>(Vector2D<T>);
impl<T: Scalar> Rect<T> {
    pub fn w(self: Self) -> T { self.0.x() }
    pub fn h(self: Self) -> T { self.0.y() }

    pub fn new(w: T, h: T) -> Self
    where
        T: PartialOrd<i32>
    {
        if w < 0 || h < 0 { panic!("Negative dimensions found for rect") }
        Self(Vector2D::new(w, h))
    }

    pub fn area(self: Self) -> T { self.w() * self.h() }
    pub fn perimeter(self: Self) -> T
    where
        T: Mul<i32, Output = T>
    { (self.w() + self.h()) * 2 }
}

impl<T: Scalar> From<Vector2D<T>> for Rect<T> {
    fn from(v: Vector2D<T>) -> Self {
        Self(v)
    }
}

impl<T: Scalar> From<Rect<T>> for Vector2D<T> {
    fn from(r: Rect<T>) -> Self {
        r.0
    }
}



#[derive(Debug, Copy, Clone, PartialEq)]
pub struct BoundingBox<T: Scalar> {
    bl: Vector2D<T>,
    tr: Vector2D<T>,
}


impl<T: Scalar> BoundingBox<T> {
    pub fn l(self: Self) -> T { self.bl.x() }
    pub fn r(self: Self) -> T { self.tr.x() }
    pub fn b(self: Self) -> T { self.bl.y() }
    pub fn t(self: Self) -> T { self.tr.y() }

    pub fn bl(self: Self) -> Vector2D<T> { self.bl }
    pub fn tr(self: Self) -> Vector2D<T> { self.tr }
    pub fn br(self: Self) -> Vector2D<T> { Vector2D::new(self.b(), self.r()) }
    pub fn tl(self: Self) -> Vector2D<T> { Vector2D::new(self.t(), self.l()) }

    pub fn from_rect(r: Rect<T>, x: T, y: T) -> Self {
        let bl = Vector2D::new(x, y);
        let tr = bl + From::<Rect<T>>::from(r);
        Self {
            bl,
            tr,
        }
    }

    pub fn from_corners(x1: T, y1: T, x2: T, y2: T) -> Self {
        if x1 > x2 || y1 > y2 { panic!("Dimension in second point was smaller than first point") }
        let bl = Vector2D::new(x1, y1);
        let tr = Vector2D::new(x2, y2);
        Self {
            bl,
            tr,
        }
    }

    pub fn contains_point(self: Self, point: Vector2D<T>) -> bool {
        (self.l()..=self.r()).contains(&point.x()) &&
        (self.b()..=self.t()).contains(&point.y())
    }

    pub fn contains_bbox(self: Self, other: Self) -> bool {
        other.l() >= self.l() &&
        other.r() <= self.r() &&
        other.b() >= self.b() &&
        other.t() <= self.t()
    }

    pub fn intersects(self: Self, other: Self) -> bool {
        !(self.r() < other.l() ||
          self.l() > other.r() ||
          self.t() < other.b() ||
          self.b() > other.t())
    }

    #[allow(unused_variables)]
    pub fn intersect(self: Self, other: Self) -> Option<Self> {
        unimplemented!();
    }
}
