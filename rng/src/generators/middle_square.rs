use crate::core::{RandomNumberGenerator, SeedableRNG, StatefulRNG};

const N_TARGET_DIGITS: usize = 14;
const N_PRODUCT_DIGITS: usize = 2 * N_TARGET_DIGITS;
const START: usize = N_PRODUCT_DIGITS / 4;
const END: usize = N_PRODUCT_DIGITS - START;

#[derive(Debug, Clone, Copy)]
pub struct MiddleSquare {
    s: u64,
}

impl SeedableRNG for MiddleSquare {
    fn new(seed: crate::core::Seed) -> Self {
        let value = (seed.val() - seed.val() % 2).max(2);
        Self { s: value }
    }
}

impl RandomNumberGenerator for MiddleSquare {
    type Bits = u64;

    fn next(self: &mut Self) -> u64 {
        let mut buff: [u8; N_PRODUCT_DIGITS] = [0x30; N_PRODUCT_DIGITS];
        let mut new_n: [u8; N_TARGET_DIGITS] = [0x30; N_TARGET_DIGITS];

        let mut n2 = (self.s as u128).pow(2);
        for byte in buff.iter_mut().rev() {
            *byte += (n2 % 10) as u8;
            n2 /= 10;
        }

        for (i, byte) in buff[START..END].iter().enumerate() {
            new_n[i] = *byte;
        }

        self.s = std::str::from_utf8(&new_n).unwrap().parse::<u64>().unwrap();
        self.s
    }
}

impl StatefulRNG for MiddleSquare {
    fn state_eq(self: &Self, other: &Self) -> bool {
        self.s == other.s
    }
    fn state_set(self: &mut Self, other: &Self) {
        self.s = other.s
    }
}
