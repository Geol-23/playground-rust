use std::{
    cmp,
    ops::{BitAnd, Shr},
};

use crate::{
    core::{RandomNumberGenerator, Seed, SeedableRNG, StatefulRNG},
    testlib::SeedGen::{Function, SampleIter},
};

#[derive(Clone, Debug)]
pub struct StatReport {
    mean: Option<Result<(), String>>,
    std: Option<Result<(), String>>,
    min: Option<Result<(), String>>,
    max: Option<Result<(), String>>,
}

impl StatReport {
    fn check_field(field: &Option<Result<(), String>>) -> bool {
        if let Some(r) = field {
            return match r {
                Ok(()) => true,
                Err(_) => false,
            };
        }
        true
    }

    pub fn check(self: &Self) -> bool {
        [&self.mean, &self.std, &self.min, &self.max]
            .iter()
            .all(|f| StatReport::check_field(f))
    }

    #[track_caller]
    pub fn assert(self: &Self, label: &str) {
        if self.check() {
            panic!("{label} failed benchmark:\n  {}", self)
        }
    }
}

impl std::fmt::Display for StatReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "mean: {:?}\n  ", self.mean);
        write!(f, "std: {:?}\n  ", self.std);
        write!(f, "min: {:?}\n  ", self.min);
        write!(f, "max: {:?}\n  ", self.max)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Bound {
    EQ(f64),
    GT(f64),
    GE(f64),
    LT(f64),
    LE(f64),
}

impl Bound {
    fn holds(self: Self, other: f64) -> bool {
        match self {
            Self::EQ(n) => n == other,
            Self::GT(n) => n < other,
            Self::GE(n) => n <= other,
            Self::LT(n) => n > other,
            Self::LE(n) => n >= other,
        }
    }
}

impl std::fmt::Display for Bound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EQ(t) => write!(f, "== {t}"),
            Self::GT(t) => write!(f, "> {t}"),
            Self::GE(t) => write!(f, ">= {t}"),
            Self::LT(t) => write!(f, "< {t}"),
            Self::LE(t) => write!(f, "<= {t}"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StatBenchmark {
    pub mean: Option<Bound>,
    pub std: Option<Bound>,
    pub min: Option<Bound>,
    pub max: Option<Bound>,
}
impl StatBenchmark {
    fn bench_field(target: Option<Bound>, real: f64) -> Option<Result<(), String>> {
        target.and_then(|b| {
            Some(
                b.holds(real)
                    .ok_or(format!("target: {}; real: {};", b, real)),
            )
        })
    }

    pub fn bench(self: &Self, other: Stats) -> StatReport {
        StatReport {
            mean: StatBenchmark::bench_field(self.mean, other.mean),
            std: StatBenchmark::bench_field(self.std, other.std),
            min: StatBenchmark::bench_field(self.min, other.min),
            max: StatBenchmark::bench_field(self.max, other.max),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    n: usize,
    mean: f64,
    std: f64,
    min: f64,
    max: f64,
}
impl Stats {
    pub fn empty() -> Self {
        Self {
            n: 0,
            mean: 0.0,
            std: 0.0,
            min: 0.0,
            max: 0.0,
        }
    }
    pub fn mix(self: Self, other: Stats) -> Self {
        let mean =
            (self.n as f64 * self.mean + other.n as f64 * other.mean) / (self.n + other.n) as f64;

        let var = (self.n as f64 * (self.std.powi(2) + self.mean.powi(2))
            + other.n as f64 * (other.std.powi(2) + other.mean.powi(2)))
            / (self.n + other.n) as f64
            - mean.powi(2);

        Stats {
            n: self.n + other.n,
            mean,
            std: var.sqrt(),
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
    pub fn add_sample<T>(self: Self, sample: T) -> Self
    where
        f64: From<T>,
    {
        let v: f64 = sample.into();
        self.mix(Stats {
            n: 1,
            mean: v,
            std: 0.0,
            min: v,
            max: v,
        })
    }
}
impl<T> From<&[T]> for Stats
where
    T: Copy + cmp::Ord,
    f64: From<T>,
{
    fn from(value: &[T]) -> Self {
        if value.len() == 0 {
            return Stats::empty();
        }

        let n = value.len() as f64;
        let min: f64 = From::from(*(value.iter().min().unwrap()));
        let max: f64 = From::from(*(value.iter().max().unwrap()));
        let mean: f64 = value.iter().map(|&x| f64::from(x)).sum::<f64>() / n;
        let variance: f64 = value
            .iter()
            .map(|&x| (f64::from(x) - mean).powi(2))
            .sum::<f64>()
            / n;

        Self {
            n: value.len(),
            mean,
            min,
            max,
            std: variance.sqrt(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Cycle {
    Found(u32, u32),
    NotWithin(u32),
}

pub struct PeriodBenchReport {
    pub mu_stats: Stats,
    pub lam_stats: Stats,
    pub n_seeds: u32,
    pub n_found: u32,
}

impl PeriodBenchReport {
    #[track_caller]
    pub fn bench(self: &Self, label: &str, mu_bench: StatBenchmark, lam_bench: StatBenchmark) {
        let mu_res = mu_bench.bench(self.mu_stats);
        let lam_res = lam_bench.bench(self.lam_stats);
        if !(mu_res.check() && lam_res.check()) {
            panic!("{label} failed benchmark:\n{}", self);
        }
    }
}

impl std::fmt::Display for PeriodBenchReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} / {} periods found\n", self.n_found, self.n_seeds);
        write!(f, "{:#?}\n{:#?}", self.mu_stats, self.lam_stats)
    }
}

pub trait RNGPeriodTest<T>: RandomNumberGenerator<T> + SeedableRNG + StatefulRNG
where
    T: cmp::Eq + cmp::Ord + Copy,
{
    fn floyds_algorithm(seed: Seed, cycle_limit: u32) -> Cycle {
        let mut cycles = 0u32;
        let mut tortise = Self::new(seed);
        tortise.next();
        let mut hare = Self::new(seed);
        hare.next();
        hare.next();

        while !tortise.state_eq(&hare) && cycles <= cycle_limit {
            cycles += 1;
            tortise.next();
            hare.next();
            hare.next();
        }

        if cycle_limit <= cycles {
            return Cycle::NotWithin(cycle_limit);
        }

        let mut mu = 0;
        let mut tortise = Self::new(seed);

        while !tortise.state_eq(&hare) {
            tortise.next();
            hare.next();
            mu += 1;
        }

        let mut lam = 1;
        tortise.next();
        hare.state_set(&tortise);
        while !tortise.state_eq(&hare) {
            hare.next();
            lam += 1;
        }

        Cycle::Found(lam, mu)
    }

    fn brents_algorithm(seed: Seed, cycle_limit: u32) -> Cycle {
        let mut cycles = 0u32;
        let mut tortise = Self::new(seed);
        let mut hare = Self::new(seed);
        hare.next();
        let mut pow: u32 = 1;
        let mut lam: u32 = 1;

        while !tortise.state_eq(&hare) && cycles <= cycle_limit {
            if pow == lam {
                tortise.state_set(&hare);
                pow *= 2;
                lam = 0;
            }
            hare.next();
            lam += 1;
            cycles += 1;
        }

        if cycle_limit <= cycles {
            return Cycle::NotWithin(cycle_limit);
        }

        let mut tortise = Self::new(seed);
        let mut hare = Self::new(seed);
        for _ in 0..lam {
            hare.next();
        }

        let mut mu = 0;
        while !tortise.state_eq(&hare) {
            hare.next();
            tortise.next();
            mu += 1;
        }

        Cycle::Found(lam, mu)
    }

    fn benchmark_period<F: Function>(seeds: SampleIter<F>, max_period: u32) -> PeriodBenchReport {
        let mut mu_stats = Stats::empty();
        let mut lam_stats = Stats::empty();
        let mut n_found: u32 = 0;
        let mut n_seeds: u32 = 0;
        for seed in seeds.map(|i| Seed::new(i)) {
            n_seeds += 1;
            match Self::brents_algorithm(seed, max_period) {
                Cycle::Found(lam, mu) => {
                    n_found += 1;
                    mu_stats = mu_stats.add_sample(mu);
                    lam_stats = lam_stats.add_sample(lam);
                }
                Cycle::NotWithin(period) => {
                    mu_stats = mu_stats.add_sample(period);
                    lam_stats = lam_stats.add_sample(period)
                }
            }
        }
        PeriodBenchReport {
            mu_stats,
            lam_stats,
            n_seeds,
            n_found,
        }
    }
}

impl<T: cmp::Eq + cmp::Ord + Copy, R: RandomNumberGenerator<T> + SeedableRNG + StatefulRNG>
    RNGPeriodTest<T> for R
{
}

fn chi2_z(counts: &[u64]) -> f64 {
    /*
     * chi2_z is the sum of the squares of the z scores of each bucket
     * z-score is the number of standard deviations from the expected value
     *
     * the bit at the end then converts the chi2 value into a z-score by applying
     * the z formula over some expected z score using the degrees of freedom i.e. how many buckets
     */
    let n_iter: u64 = counts.iter().sum();
    let n_buckets = counts.len();
    let e = n_iter as f64 / n_buckets as f64;

    let chi2: f64 = counts
        .iter()
        .map(|&c| {
            let d = c as f64 - e;
            d * d / e
        })
        .sum();

    // the bit at the end
    let df = (n_buckets - 1) as f64;
    (chi2 - df) / (2.0 * df).sqrt()
}

pub struct BiasAccumulator<T> {
    bits: u64,           // Width of T
    n: u64,              //
    bits_hi: [u64; 256], // Upper 8 bits
    bits_lo: [u64; 256], // Lower 8 bits
    pairs: [u64; 256],   // 4 bits from previous then 4 bits from current
    full: Vec<u64>,      // Bias for all bits
    prev: Option<T>,
}
impl<T> BiasAccumulator<T>
where
    T: Shr<u64, Output = T> + Copy,
    u64: From<T>,
{
    fn new() -> Self {
        let bits = (std::mem::size_of::<T>() * 8) as u64;
        Self {
            bits: bits,
            n: 0,
            bits_hi: [0; 256],
            bits_lo: [0; 256],
            pairs: [0; 256],
            full: vec![0; bits as usize],
            prev: None,
        }
    }
    fn push(self: &mut Self, x: T) {
        self.n += 1;
        self.bits_hi[<u64>::from(x >> (self.bits - 8)) as usize] += 1;
        self.bits_lo[(<u64>::from(x) & 0xFF) as usize] += 1;

        for b in 0..self.bits {
            self.full[b as usize] += (<u64>::from(x >> b) & 1) as u64;
        }

        match self.prev.take() {
            None => self.prev = Some(x),
            Some(prev) => {
                self.pairs[((<u64>::from(prev) & 0xF) << 4 | <u64>::from(x) & 0xF) as usize] += 1;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BiasReport {
    pub high_bits: f64,
    pub low_bits: f64,
    pub low_pairs: f64,
    pub worst_bit: f64, // max |z| over the 64 bit-balance tests
}

impl<T> From<BiasAccumulator<T>> for BiasReport {
    fn from(value: BiasAccumulator<T>) -> Self {
        let sd = (value.n as f64).sqrt() / 2.0;
        let worst_bit = value
            .full
            .iter()
            .map(|&c| ((c as f64 - value.n as f64 / 2.0) / sd).abs())
            .fold(0.0, f64::max);
        Self {
            high_bits: chi2_z(&value.bits_hi),
            low_bits: chi2_z(&value.bits_lo),
            low_pairs: chi2_z(&value.pairs),
            worst_bit,
        }
    }
}

pub trait RNGBiasTest<T>: RandomNumberGenerator<T> + SeedableRNG
where
    T: Shr<u64, Output = T> + Copy,
    u64: From<T>,
{
    fn benchmark_bias<F: Function>(seeds: SampleIter<F>, n_samples: u64) -> BiasReport {
        let mut accumulator = BiasAccumulator::new();
        for seed in seeds.map(|i| Seed::new::<u64>(i)) {
            Self::new(seed)
                .iter_mut()
                .take(n_samples as usize)
                .fold(&mut accumulator, |a, i| {
                    a.push(i);
                    a
                });
        }
        <BiasReport>::from(accumulator)
    }
}

impl<T: Shr<u64, Output = T> + Copy, R: RandomNumberGenerator<T> + SeedableRNG> RNGBiasTest<T> for R where
    u64: From<T>
{
}

pub mod SeedGen {
    use std::f64;

    pub trait Function: Sized {
        fn at(self: &Self, i: u64) -> u64;
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Linear {
        a: u64,
        b: u64,
    }
    impl Linear {
        pub const UNIT: Self = Self { a: 1, b: 0 };
        const fn new(a: u64, b: u64) -> Self {
            Self { a, b }
        }
    }
    impl Function for Linear {
        fn at(self: &Self, i: u64) -> u64 {
            self.a.wrapping_mul(i).wrapping_add(self.b)
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Quadratic {
        a: u64,
        b: u64,
        c: u64,
    }
    impl Quadratic {
        const UNIT: Self = Self { a: 1, b: 0, c: 0 };
        fn new(a: u64, b: u64, c: u64) -> Self {
            Self { a, b, c }
        }
    }
    impl Function for Quadratic {
        fn at(self: &Self, i: u64) -> u64 {
            self.a
                .wrapping_mul(i.wrapping_pow(2))
                .wrapping_add(self.b.wrapping_mul(i))
                .wrapping_add(self.c)
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Exponential {
        a: u64,
        b: u64,
    }
    impl Exponential {
        const UNIT: Self = Self { a: 0, b: u64::MAX };
        fn new(a: u64, b: u64) -> Self {
            Self { a, b }
        }
    }
    impl Function for Exponential {
        fn at(self: &Self, i: u64) -> u64 {
            1u64.wrapping_mul(((i.wrapping_add(self.a)) as f64).exp() as u64)
                .wrapping_add(self.b)
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Decay {
        a: u64,
        b: u64,
    }
    impl Decay {
        const UNIT: Self = Self { a: 44, b: 0 };
        fn new(a: u64, b: u64) -> Self {
            Self { a, b }
        }
    }
    impl Function for Decay {
        fn at(self: &Self, i: u64) -> u64 {
            1u64.wrapping_div(((i.wrapping_add(self.a)) as f64).exp() as u64)
                .wrapping_add(self.b)
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Sampler {
        start: u64,
        stop: u64,
        n: u64,
    }

    impl Sampler {
        pub const fn new(n: u64, start: u64, stop: u64) -> Self {
            Self { start, stop, n }
        }
        pub fn apply<F: Function>(self: Self, f: F) -> SampleIter<F> {
            SampleIter::new(f, self)
        }
        pub fn sample(self: &Self, i: u64) -> u64 {
            let span = self.start.abs_diff(self.stop); // u64, always >= 0
            let off = (span as u128 * i as u128 / self.n as u128) as u64;
            if self.stop >= self.start {
                self.start + off
            } else {
                self.start - off
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub struct SampleIter<F> {
        f: F,
        s: Sampler,
        c: u64,
    }

    impl<F: Function> SampleIter<F> {
        fn new(f: F, s: Sampler) -> Self {
            Self { f, s, c: 0 }
        }
    }

    impl<F: Function> Iterator for SampleIter<F> {
        type Item = u64;
        fn next(&mut self) -> Option<Self::Item> {
            if self.c >= self.s.n {
                return None;
            }

            let v = self.f.at(self.s.sample(self.c));
            self.c += 1;

            Some(v)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Sampler;
        const slimit: Sampler = Sampler::new(3, 0, u64::MAX);

        #[cfg(test)]
        mod linear {
            use super::super::{Linear, Sampler};
            use super::slimit;

            const asc: Linear = Linear::UNIT;
            const desc: Linear = Linear::new(u64::MAX, 10);

            #[test]
            fn asc_basic() {
                let samples: Vec<u64> = Sampler::new(10, 0, 10).apply(asc).collect();
                dbg!(&samples);

                assert_eq!(samples[0], 0);
                assert_eq!(samples[5], 5);
                assert_eq!(samples[9], 9);
            }

            #[test]
            fn asc_limit() {
                let samples: Vec<u64> = slimit.apply(asc).collect();
                dbg!(&samples);

                assert_eq!(samples[0], 0);
                assert_eq!(samples[1], u64::MAX / 3);
                assert_eq!(samples[2], u64::MAX / 3 * 2);
            }

            #[test]
            fn desc_basic() {
                let samples: Vec<u64> = Sampler::new(10, 0, 10).apply(desc).collect();
                dbg!(&samples);

                assert_eq!(samples[0], 10);
                assert_eq!(samples[5], 5);
                assert_eq!(samples[9], 1);
            }

            #[test]
            fn desc_limit() {
                let samples: Vec<u64> = slimit.apply(Linear::new(u64::MAX, u64::MAX)).collect();
                dbg!(&samples);

                assert_eq!(samples[0], u64::MAX);
                assert_eq!(samples[1], u64::MAX / 3 * 2);
                assert_eq!(samples[2], u64::MAX / 3);
            }
        }

        #[cfg(test)]
        mod quadratic {
            use super::super::{Quadratic, Sampler};
            use super::slimit;

            #[test]
            fn quadratic_basic() {
                let samples: Vec<u64> = Sampler::new(10, 0, 10).apply(Quadratic::UNIT).collect();
                dbg!(&samples);

                assert_eq!(samples, vec![0, 1, 4, 9, 16, 25, 36, 49, 64, 81]);
            }

            #[test]
            fn quadratic_limit() {
                let samples: Vec<u64> = slimit.apply(Quadratic::UNIT).collect();
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
}
