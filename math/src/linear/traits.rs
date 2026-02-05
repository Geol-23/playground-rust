pub trait Lerp<Other = Self> {
    type Output;

    fn lerp(self, other: Other, t: f64) -> Self::Output;
}

macro_rules! impl_lerp {
    ($($scalar:ty),+ $(,)?) => {
        $(
            impl Lerp for $scalar
            where
                f64: From<$scalar>
            {
                type Output = f64;

                fn lerp(self, other: Self, t: f64) -> Self::Output {
                    let a: f64 = self.into();
                    let b: f64 = other.into();
                    a + (b - a) * t
                }
            }
        )+
    };
}
impl_lerp!(u8, u16, u32, i8, i16, i32, f32, f64);
