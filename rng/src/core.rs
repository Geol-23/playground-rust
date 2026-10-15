use std::default::Default;
use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug)]
pub struct Seed(pub u64);
impl Seed {
    fn from_time() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x9E37_79B9_7F4A_7C15),
        )
    }
}
impl Default for Seed {
    fn default() -> Self {
        Self::from_time()
    }
}

pub struct RandomNumberIterator<R, T> {
    rng: R,
    _marker: PhantomData<T>,
}
impl<R, T> Iterator for RandomNumberIterator<R, T>
where
    R: RandomNumberGenerator<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        Some(RandomNumberGenerator::next(&mut self.rng))
    }
}

pub trait RandomNumberGenerator<T>: Sized {
    fn new(seed: Seed) -> Self;
    fn next(self: &mut Self) -> T;
    fn from_time() -> Self {
        Self::new(Seed::from_time())
    }
    fn iter_mut(self: &mut Self) -> RandomNumberIterator<&mut Self, T> {
        RandomNumberIterator {
            rng: self,
            _marker: PhantomData,
        }
    }
}
