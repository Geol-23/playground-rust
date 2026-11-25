use crate::core::{RandomNumberGenerator, Seed, SeedableRNG, StatefulRNG};

#[derive(Debug, Clone, Copy)]
pub struct Lehmer<const A: u64, const M: u64> {
    state: u64,
}

impl<const A: u64, const M: u64> SeedableRNG for Lehmer<A, M> {
    fn new(seed: Seed) -> Self {
        let s0 = seed.val() % M;
        Self { state: s0.max(1) }
    }
}

impl<const A: u64, const M: u64> RandomNumberGenerator for Lehmer<A, M> {
    type Bits = u64;

    fn next(self: &mut Self) -> u64 {
        self.state = ((A as u128 * self.state as u128) % M as u128) as u64;
        self.state
    }
}

impl<const A: u64, const M: u64> StatefulRNG for Lehmer<A, M> {
    fn state_eq(self: &Self, other: &Self) -> bool {
        self.state == other.state
    }
    fn state_set(self: &mut Self, other: &Self) {
        self.state = other.state
    }
}

macro_rules! variation {
    ( $( $name:ident: ($a:expr, $m:expr) ),+ $(,)? ) => {
        $(
            pub type $name = Lehmer<{ $a }, { $m }>;
        )*
    };
}

variation!(
    Minstd: (7u64.pow(5), (1u64 << 31) - 1),
    Zx81: (75, 2u64.pow(16) + 1),
    Ranf: (44_485_709_377_909, 2u64.pow(48)),
    Randu: (65_539, 2u64.pow(31)),
);
