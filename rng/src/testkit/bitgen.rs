use std::marker::PhantomData;

// Function definitions ---------------------------------------------------------------------------
pub trait Function {
    fn at(i: u64) -> u64;
}

#[macro_export]
macro_rules! count {
    () => {
        0usize
    };
    ($_t:tt $($t:tt)*) => {
        1usize + count!($($t)*)
    };
}

macro_rules! make_function {
    ( $typename:ident( $($coeff:ident),+ $(,)? ): |$i:ident| $alg:expr ) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $typename<$( const $coeff: u64, )+> { }

        impl<$( const $coeff: u64, )+> Function for $typename<$( $coeff, )+> {
            #[allow(unused_variables)]
            fn at($i: u64) -> u64 {
                $alg
            }
        }
    };
}

make_function!(Linear(A, B): |i|
    A
        .wrapping_mul(i)
        .wrapping_add(B)
);

make_function!(Quadratic(A, B, C): |i|
    A
        .wrapping_mul(i.wrapping_pow(2))
        .wrapping_add(B.wrapping_mul(i))
        .wrapping_add(C)
);

make_function!(Exponential(A, B): |i|
    1u64.wrapping_mul(((i.wrapping_add(A)) as f64).exp() as u64)
        .wrapping_add(B)
);

make_function!(Decay(A, B): |i|
    1u64.wrapping_div(((i.wrapping_add(A)) as f64).exp() as u64)
        .wrapping_add(B)
);

// Sampler definitions ----------------------------------------------------------------------------
pub trait SamplerTrait: Sized {
    fn apply<F: Function>() -> SampleIter<F, Self>;
    fn sample(i: u64) -> u64;
    fn samples() -> u64;
}

#[derive(Debug, Clone, Copy)]
pub struct Sampler<const N: u64, const START: u64, const STOP: u64> {}

impl<const N: u64, const START: u64, const STOP: u64> SamplerTrait for Sampler<N, START, STOP> {
    fn apply<F: Function>() -> SampleIter<F, Self> {
        SampleIter::new()
    }
    fn sample(i: u64) -> u64 {
        let span = START.abs_diff(STOP); // u64, always >= 0
        let off = (span as u128 * i as u128 / N as u128) as u64;
        if STOP >= START {
            START + off
        } else {
            START - off
        }
    }
    fn samples() -> u64 {
        N
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SampleIter<F, S> {
    _f: PhantomData<F>,
    _s: PhantomData<S>,
    c: u64,
}

impl<F: Function, S: SamplerTrait> SampleIter<F, S> {
    pub fn new() -> Self {
        Self {
            _f: PhantomData,
            _s: PhantomData,
            c: 0,
        }
    }
}

impl<F: Function, S: SamplerTrait> Iterator for SampleIter<F, S> {
    type Item = u64;
    fn next(&mut self) -> Option<Self::Item> {
        if self.c >= S::samples() {
            return None;
        }

        let v = F::at(S::sample(self.c));
        self.c += 1;

        Some(v)
    }
}

// Tests ------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::Sampler;
    type SLimit = Sampler<3, 0, { u64::MAX }>;

    #[cfg(test)]
    mod linear {
        use super::super::{Linear, Sampler, SamplerTrait};
        use super::SLimit;

        type Asc = Linear<1, 0>;
        type Desc = Linear<{ u64::MAX }, 10>;

        #[test]
        fn asc_basic() {
            let samples: Vec<u64> = Sampler::<10, 0, 10>::apply::<Asc>().collect();
            dbg!(&samples);

            assert_eq!(samples[0], 0);
            assert_eq!(samples[5], 5);
            assert_eq!(samples[9], 9);
        }

        #[test]
        fn asc_limit() {
            let samples: Vec<u64> = SLimit::apply::<Asc>().collect();
            dbg!(&samples);

            assert_eq!(samples[0], 0);
            assert_eq!(samples[1], u64::MAX / 3);
            assert_eq!(samples[2], u64::MAX / 3 * 2);
        }

        #[test]
        fn desc_basic() {
            let samples: Vec<u64> = Sampler::<10, 0, 10>::apply::<Desc>().collect();
            dbg!(&samples);

            assert_eq!(samples[0], 10);
            assert_eq!(samples[5], 5);
            assert_eq!(samples[9], 1);
        }

        #[test]
        fn desc_limit() {
            let samples: Vec<u64> = SLimit::apply::<Linear<{ u64::MAX }, { u64::MAX }>>().collect();
            dbg!(&samples);

            assert_eq!(samples[0], u64::MAX);
            assert_eq!(samples[1], u64::MAX / 3 * 2);
            assert_eq!(samples[2], u64::MAX / 3);
        }
    }

    #[cfg(test)]
    mod quadratic {
        use super::super::{Quadratic, Sampler, SamplerTrait};
        use super::SLimit;

        #[test]
        fn quadratic_basic() {
            let samples: Vec<u64> = Sampler::<10, 0, 10>::apply::<Quadratic<1, 0, 0>>().collect();
            dbg!(&samples);

            assert_eq!(samples, vec![0, 1, 4, 9, 16, 25, 36, 49, 64, 81]);
        }

        #[test]
        fn quadratic_limit() {
            let samples: Vec<u64> = SLimit::apply::<Quadratic<1, 0, 0>>().collect();
            let target = vec![
                0,
                (u64::MAX.wrapping_div(3)).wrapping_pow(2),
                (u64::MAX.wrapping_div(3).wrapping_mul(2)).wrapping_pow(2),
            ];
            dbg!(&samples);
            dbg!(&target);

            assert_eq!(samples, target)
        }
    }
}
