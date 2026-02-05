pub mod geo;
pub mod linear;
pub mod traits;


#[cfg(test)]
mod tests {
    use super::*;

    mod vector {
        use super::*;

        #[test]
        fn add_00_00() {
            let p1 = linear::Vec2::new(0, 0);
            let p2 = linear::Vec2::new(0, 0);
            let p3 = linear::Vec2::new(0, 0);
            assert_eq!(p1 + p2, p3);
        }

        #[test]
        fn add_11_11() {
            let p1 = linear::Vec2::new(1, 1);
            let p2 = linear::Vec2::new(1, 1);
            let p3 = linear::Vec2::new(2, 2);
            assert_eq!(p1 + p2, p3);
        }

        #[test]
        fn add_n1n1_11() {
            let p1 = linear::Vec2::new(-1, -1);
            let p2 = linear::Vec2::new(1, 1);
            let p3 = linear::Vec2::new(0, 0);
            assert_eq!(p1 + p2, p3);
        }

        #[test]
        fn add_11_11_u8() {
            let p1 = linear::Vec2::new(1u8, 1u8);
            let p2 = linear::Vec2::new(1u8, 1u8);
            let p3 = linear::Vec2::new(2u8, 2u8);
            assert_eq!(p1 + p2, p3);
        }

        #[test]
        fn vector_dot() {
            let p1 = linear::Vec2::new(1, 1);
            let p2 = linear::Vec2::new(1, 1);
            let p3 = 2;
            assert_eq!((&p1).dot(&p2), p3);
        }
    }

    mod rect {
        use super::*;
        #[test]
        fn basic() {
            let r1 = geo::Rect::new(10, 20);
            assert_eq!(r1.area(), 200);
            assert_eq!(r1.perimeter(), 60);
        }

    }

    mod bounding_box {
        use super::*;

        #[test]
        fn intersect_full() {
            let b1 = geo::BoundingBox::from_corners(0, 0, 10, 10);
            let b2 = geo::BoundingBox::from_corners(2, 2, 5, 5);
            assert!(b1.intersects(b2));
            assert!(b2.intersects(b1));
        }

        #[test]
        fn intersect_partial() {
            let b1 = geo::BoundingBox::from_corners(0, 0, 2, 2);
            let b2 = geo::BoundingBox::from_corners(1, 1, 3, 3);
            assert!(b1.intersects(b2));
            assert!(b2.intersects(b1));

            let b1 = geo::BoundingBox::from_corners(0, 0, 2, 2);
            let b2 = geo::BoundingBox::from_corners(-1, -1, 3, 3);
            assert!(b1.intersects(b2));
            assert!(b2.intersects(b1));
        }

        #[test]
        fn intersect_corner() {
            let b1 = geo::BoundingBox::from_corners(0, 0, 2, 2);
            let b2 = geo::BoundingBox::from_corners(2, 2, 3, 3);
            assert!(b1.intersects(b2));
            assert!(b2.intersects(b1));
        }

        #[test]
        fn intersect_none() {
            let b1 = geo::BoundingBox::from_corners(0, 0, 2, 2);
            let b2 = geo::BoundingBox::from_corners(3, 3, 4, 4);
            assert!(!b1.intersects(b2));
            assert!(!b2.intersects(b1));
        }

        #[test]
        fn intersect_perfect() {
            let b1 = geo::BoundingBox::from_corners(0, 0, 2, 2);
            let b2 = geo::BoundingBox::from_corners(0, 0, 2, 2);
            assert!(b1.intersects(b2));
            assert!(b2.intersects(b1));
        }
    }

    mod lerp {
        use super::*;
        use linear::Lerp;
        #[test]
        fn basic() {
            let a = 10;
            let b = 20;
            assert_eq!(a.lerp(b, 0.5), 15.0)
        }

        #[test]
        fn vector() {
            let v1 = linear::Vec2::new(0, 0);
            let v2 = linear::Vec2::new(5, 15);
            let v3 = linear::Vec2::new(2.5, 7.5);
            assert_eq!(v1.lerp(v2, 0.5), v3);
        }
    }
}
