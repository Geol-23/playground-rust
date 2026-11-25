pub mod bitgen;
pub mod framework;
pub mod suite;

use std::{
    cmp,
    hint::black_box,
    ops::Shr,
    time::{Duration, Instant},
};

use crate::{
    core::{RandomNumberGenerator, Seed, SeedableRNG, StatefulRNG},
    testkit::bitgen::{Function, SampleIter, SamplerTrait},
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
        write!(f, "mean: {:?}\n  ", self.mean)?;
        write!(f, "std: {:?}\n  ", self.std)?;
        write!(f, "min: {:?}\n  ", self.min)?;
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

// Benchmark — Period -----------------------------------------------------------------------------
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
        write!(f, "{} / {} periods found\n", self.n_found, self.n_seeds)?;
        write!(f, "{:#?}\n{:#?}", self.mu_stats, self.lam_stats)
    }
}

pub trait RNGPeriodTest: RandomNumberGenerator + SeedableRNG + StatefulRNG {
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

    fn benchmark_period<F, S>(seeds: SampleIter<F, S>, max_period: u32) -> PeriodBenchReport
    where
        F: Function,
        S: SamplerTrait,
    {
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

impl<R: RandomNumberGenerator + SeedableRNG + StatefulRNG> RNGPeriodTest for R {}

// Benchmark — Bias -------------------------------------------------------------------------------
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

pub trait RNGBiasTest: RandomNumberGenerator + SeedableRNG
where
    Self::Bits: Shr<u64, Output = Self::Bits> + Copy,
    u64: From<Self::Bits>,
{
    fn benchmark_bias<F, S>(seeds: SampleIter<F, S>, n_samples: u64) -> BiasReport
    where
        F: Function,
        S: SamplerTrait,
    {
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
        accumulator.into()
    }
}

impl<R: RandomNumberGenerator + SeedableRNG> RNGBiasTest for R
where
    Self::Bits: Shr<u64, Output = Self::Bits> + Copy,
    u64: From<Self::Bits>,
{
}

// Benchmark — Speed ------------------------------------------------------------------------------
#[derive(Debug, Clone, Copy)]
pub struct SpeedReport {
    n: u64,
    t: Duration,
    s: u128,
}

impl SpeedReport {
    fn new() -> Self {
        SpeedReport {
            n: 0,
            t: Duration::new(0, 0),
            s: 0,
        }
    }
    fn push(self: &mut Self, n: u64, t: Duration) {
        self.n += n;
        self.t += t;
        self.s = self.t.as_nanos() / (self.n as u128);
    }
}

impl std::fmt::Display for SpeedReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SpeedReport {{\n")?;
        write!(f, "    n: {}\n", self.n)?;
        write!(f, "    t: {:?}\n", self.t)?;
        write!(f, "    s: {}ns\n", self.s)?;
        write!(f, "}}\n")
    }
}

pub trait RNGSpeedTest: RandomNumberGenerator + SeedableRNG {
    fn benchmark_speed<F, S>(seeds: SampleIter<F, S>, n_samples: u64) -> SpeedReport
    where
        F: Function,
        S: SamplerTrait,
    {
        let mut report = SpeedReport::new();
        for seed in seeds.map(|i| Seed::new::<u64>(i)) {
            let mut rng = Self::new(seed);

            // Warmup
            for _ in 0..10 {
                black_box(rng.next());
            }

            // Run
            let t1 = Instant::now();
            for _ in 0..n_samples {
                black_box(rng.next());
            }
            let t2 = Instant::now() - t1;

            black_box(rng);
            report.push(n_samples, t2);
        }
        report
    }
}

impl<R: RandomNumberGenerator + SeedableRNG> RNGSpeedTest for R {}
