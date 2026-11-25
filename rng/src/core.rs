use std::default::Default;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug)]
pub struct Seed(u64);
impl Seed {
    fn from_time() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x9E37_79B9_7F4A_7C15),
        )
    }
    pub const fn val(self: Self) -> u64 {
        self.0
    }
    pub fn new<T>(other: T) -> Self
    where
        u64: From<T>,
    {
        Self(u64::from(other))
    }
}
impl Default for Seed {
    fn default() -> Self {
        Self::from_time()
    }
}

pub struct RandomNumberIterator<R> {
    rng: R,
}
impl<R> Iterator for RandomNumberIterator<R>
where
    R: RandomNumberGenerator,
{
    type Item = R::Bits;
    fn next(&mut self) -> Option<Self::Item> {
        Some(RandomNumberGenerator::next(&mut self.rng))
    }
}

pub trait RandomNumberGenerator: Sized {
    type Bits;

    fn next(self: &mut Self) -> Self::Bits;
    fn iter_mut(self: &mut Self) -> RandomNumberIterator<&mut Self>
    where
        Self: Sized,
    {
        RandomNumberIterator { rng: self }
    }
}

pub trait SeedableRNG: Sized {
    fn new(seed: Seed) -> Self;
    fn from_time() -> Self {
        Self::new(Seed::from_time())
    }
}

impl<R: RandomNumberGenerator + Sized> RandomNumberGenerator for &mut R {
    type Bits = R::Bits;

    fn next(self: &mut Self) -> R::Bits {
        (**self).next()
    }
}

pub trait StatefulRNG: Sized + Clone {
    fn state_set(self: &mut Self, other: &Self);
    fn state_eq(self: &Self, other: &Self) -> bool;
}
