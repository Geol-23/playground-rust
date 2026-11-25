use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use crate::{
    core::{RandomNumberGenerator, StatefulRNG},
    testkit::framework::{
        Benchmark, Report, ReportAccumulator, ReportSummary, RunBenchmark, Stats, Value,
    },
};

// Benchmark — Period -----------------------------------------------------------------------------
type Lam = usize;
type Mu = usize;

#[derive(Debug, Copy, Clone)]
pub enum ReportPeriod<const N: usize> {
    Found(Lam, Mu),
    NotWithin,
}

impl<const N: usize> Report for ReportPeriod<N> {}

#[derive(Debug, Clone, Copy)]
pub struct ReportAccPeriod<const N: usize> {
    mu_stats: Stats,
    lam_stats: Stats,
    n_seeds: usize,
    n_found: usize,
}

impl<const N: usize> ReportAccumulator for ReportAccPeriod<N> {
    type Report = ReportPeriod<N>;
    type ReportSummary = ReportSumPeriod<N>;

    fn new() -> Self {
        Self {
            mu_stats: Stats::empty(),
            lam_stats: Stats::empty(),
            n_seeds: 0,
            n_found: 0,
        }
    }

    fn mix(&mut self, other: &Self::Report) {
        match other {
            ReportPeriod::NotWithin => self.n_seeds += 1,
            ReportPeriod::Found(lam, mu) => {
                self.mu_stats = self.mu_stats.add_sample(u32::try_from(*mu).unwrap());
                self.lam_stats = self.lam_stats.add_sample(u32::try_from(*lam).unwrap());
                self.n_seeds += 1;
                self.n_found += 1;
            }
        }
    }

    fn summary(&self) -> Self::ReportSummary {
        Self::ReportSummary {
            mu_stats: self.mu_stats,
            lam_stats: self.lam_stats,
            n_seeds: self.n_seeds,
            n_found: self.n_found,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReportSumPeriod<const N: usize> {
    pub mu_stats: Stats,
    pub lam_stats: Stats,
    pub n_seeds: usize,
    pub n_found: usize,
}

impl<const N: usize> ReportSummary for ReportSumPeriod<N> {
    const N_FIELDS: usize = 9;
    type Fields = [(&'static str, Value); 9];
    fn fields(&self) -> [(&'static str, Value); 9] {
        [
            ("mu_mean", self.mu_stats.mean.into()),
            ("mu_std", self.mu_stats.std.into()),
            ("mu_min", self.mu_stats.min.into()),
            ("mu_max", self.mu_stats.max.into()),
            ("lam_mean", self.lam_stats.mean.into()),
            ("lam_std", self.lam_stats.std.into()),
            ("lam_min", self.lam_stats.min.into()),
            ("lam_max", self.lam_stats.max.into()),
            ("n_found", self.n_found.into()),
        ]
    }
}

#[allow(dead_code)]
fn floyds_algorithm<R, const N: usize>(rng: &R) -> ReportPeriod<N>
where
    R: RandomNumberGenerator + StatefulRNG,
{
    let mut cycles = 0usize;
    let mem = rng;

    let mut tortise = rng.clone();
    tortise.next();
    let mut hare = rng.clone();
    hare.next();
    hare.next();

    while !tortise.state_eq(&hare) && cycles <= N {
        cycles += 1;
        tortise.next();
        hare.next();
        hare.next();
    }

    if N < cycles {
        return ReportPeriod::NotWithin;
    }

    let mut mu = 0;
    let mut tortise = mem.clone();

    while !tortise.state_eq(&hare) {
        tortise.next();
        hare.next();
        mu += 1;
    }

    let mut lam = 1;
    hare.state_set(&tortise);
    tortise.next();
    while !tortise.state_eq(&hare) {
        hare.next();
        lam += 1;
    }

    ReportPeriod::Found(lam, mu)
}

fn brents_algorithm<R, const N: usize>(rng: &R) -> ReportPeriod<N>
where
    R: RandomNumberGenerator + StatefulRNG,
{
    let mut cycles = 0usize;
    let mem = rng;

    let mut tortise = mem.clone();
    let mut hare = mem.clone();
    hare.next();

    let mut pow: usize = 1;
    let mut lam: usize = 1;

    while !tortise.state_eq(&hare) && cycles <= N {
        if pow == lam {
            tortise.state_set(&hare);
            pow *= 2;
            lam = 0;
        }
        hare.next();
        lam += 1;
        cycles += 1;
    }

    if N < cycles {
        return ReportPeriod::NotWithin;
    }

    let mut tortise = mem.clone();
    let mut hare = mem.clone();
    for _ in 0..lam {
        hare.next();
    }

    let mut mu = 0;
    while !tortise.state_eq(&hare) {
        hare.next();
        tortise.next();
        mu += 1;
    }

    ReportPeriod::Found(lam, mu)
}

#[derive(Default)]
pub struct BenchmarkPeriod<const N: usize> {}

impl<const N: usize> Benchmark for BenchmarkPeriod<N> {
    const NAME: &'static str = "BenchmarkPeriod";
    type Report = ReportPeriod<N>;
    type ReportAccumulator = ReportAccPeriod<N>;
}

impl<const N: usize, R> RunBenchmark<R> for BenchmarkPeriod<N>
where
    R: RandomNumberGenerator + StatefulRNG,
{
    fn bench(&self, rng: &mut R) -> Self::Report {
        brents_algorithm(rng)
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct ReportBias<const N: usize> {
    high_bits: f64,
    low_bits: f64,
    low_pairs: f64,
    worst_bit: f64,
}

#[derive(Debug, Clone)]
struct _BiasAccumulator<const N: usize> {
    bits_hi: [u64; 256], // Upper 8 bits
    bits_lo: [u64; 256], // Lower 8 bits
    pairs: [u64; 256],   // 4 bits from previous then 4 bits from current
    full: Vec<u64>,      // Bias for all bits
    prev: Option<u8>,
}

impl<const N: usize> _BiasAccumulator<N> {
    fn new() -> Self {
        Self {
            bits_hi: [0; 256],
            bits_lo: [0; 256],
            pairs: [0; 256],
            full: Vec::new(),
            prev: None,
        }
    }

    fn push<T>(&mut self, x: T)
    where
        T: Copy + Into<u64>,
    {
        let b: usize = std::mem::size_of::<T>() * 8;
        let x: u64 = x.into();
        self.bits_hi[(x >> (b - 8)) as usize] += 1;
        self.bits_lo[(x & 0xFF) as usize] += 1;

        if self.full.len() != b {
            self.full.resize(b, 0);
        }

        for i in 0..b {
            self.full[i as usize] += (<u64>::from(x >> i) & 1) as u64;
        }

        match self.prev.take() {
            None => self.prev = Some((x & 0xF) as u8),
            Some(prev) => {
                self.pairs[((prev as u64 & 0xF) << 4 | (x as u64 & 0xF)) as usize] += 1;
            }
        }
    }
}
impl<const N: usize> From<_BiasAccumulator<N>> for ReportBias<N> {
    fn from(value: _BiasAccumulator<N>) -> Self {
        let sd = (N as f64).sqrt() / 2.0;
        let worst_bit = value
            .full
            .iter()
            .map(|&c| ((c as f64 - N as f64 / 2.0) / sd).abs())
            .fold(0.0, f64::max);
        Self {
            high_bits: chi2_z(&value.bits_hi),
            low_bits: chi2_z(&value.bits_lo),
            low_pairs: chi2_z(&value.pairs),
            worst_bit,
        }
    }
}

impl<const N: usize> Report for ReportBias<N> {}

#[derive(Debug, Clone, Copy)]
pub struct ReportAccBias<const N: usize> {
    high_bits: Stats,
    low_bits: Stats,
    low_pairs: Stats,
    worst_bit: Stats,
    n_seeds: usize,
}

impl<const N: usize> ReportAccumulator for ReportAccBias<N> {
    type Report = ReportBias<N>;
    type ReportSummary = ReportSumBias<N>;

    fn mix(&mut self, other: &Self::Report) {
        self.high_bits = self.high_bits.add_sample(other.high_bits);
        self.low_bits = self.low_bits.add_sample(other.low_bits);
        self.low_pairs = self.low_pairs.add_sample(other.low_pairs);
        self.worst_bit = self.worst_bit.add_sample(other.worst_bit);
        self.n_seeds += 1;
    }

    fn new() -> Self {
        Self {
            high_bits: Stats::empty(),
            low_bits: Stats::empty(),
            low_pairs: Stats::empty(),
            worst_bit: Stats::empty(),
            n_seeds: 0usize,
        }
    }

    fn summary(&self) -> Self::ReportSummary {
        ReportSumBias {
            high_bits: self.high_bits,
            low_bits: self.low_bits,
            low_pairs: self.low_pairs,
            worst_bit: self.worst_bit,
            n_seeds: self.n_seeds,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReportSumBias<const N: usize> {
    pub high_bits: Stats,
    pub low_bits: Stats,
    pub low_pairs: Stats,
    pub worst_bit: Stats,
    pub n_seeds: usize,
}

impl<const N: usize> ReportSummary for ReportSumBias<N> {
    const N_FIELDS: usize = 16;
    type Fields = [(&'static str, Value); 16];
    fn fields(&self) -> [(&'static str, Value); 16] {
        [
            ("high_bits_mean", self.high_bits.mean.into()),
            ("high_bits_std", self.high_bits.std.into()),
            ("high_bits_min", self.high_bits.min.into()),
            ("high_bits_max", self.high_bits.max.into()),
            ("low_bits_mean", self.low_bits.mean.into()),
            ("low_bits_std", self.low_bits.std.into()),
            ("low_bits_min", self.low_bits.min.into()),
            ("low_bits_max", self.low_bits.max.into()),
            ("low_pairs_mean", self.low_pairs.mean.into()),
            ("low_pairs_std", self.low_pairs.std.into()),
            ("low_pairs_min", self.low_pairs.min.into()),
            ("low_pairs_max", self.low_pairs.max.into()),
            ("worst_bit_mean", self.worst_bit.mean.into()),
            ("worst_bit_std", self.worst_bit.std.into()),
            ("worst_bit_min", self.worst_bit.min.into()),
            ("worst_bit_max", self.worst_bit.max.into()),
        ]
    }
}

#[derive(Default)]
pub struct BenchmarkBias<const N: usize> {}

impl<const N: usize> Benchmark for BenchmarkBias<N> {
    const NAME: &'static str = "BenchmarkBias";
    type Report = ReportBias<N>;
    type ReportAccumulator = ReportAccBias<N>;
}

impl<const N: usize, R> RunBenchmark<R> for BenchmarkBias<N>
where
    R: RandomNumberGenerator,
    R::Bits: Copy + Into<u64>,
{
    fn bench(&self, rng: &mut R) -> Self::Report {
        let mut report: _BiasAccumulator<N> = _BiasAccumulator::new();
        rng.iter_mut().take(N).fold(&mut report, |a, i| {
            a.push(i);
            a
        });
        report.into()
    }
}

// Benchmark — Speed ------------------------------------------------------------------------------
#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct ReportSpeed<const N: usize> {
    t: Duration,
    s: u128,
}

impl<const N: usize> Report for ReportSpeed<N> {}

#[derive(Debug, Clone, Copy)]
pub struct ReportAccSpeed<const N: usize> {
    speed: Stats,
}

impl<const N: usize> ReportAccumulator for ReportAccSpeed<N> {
    type Report = ReportSpeed<N>;
    type ReportSummary = ReportSumSpeed<N>;

    fn mix(&mut self, other: &Self::Report) {
        self.speed = self
            .speed
            .add_sample((other.t / u32::try_from(N).unwrap()).as_secs_f64());
    }
    fn new() -> Self {
        Self {
            speed: Stats::empty(),
        }
    }
    fn summary(&self) -> Self::ReportSummary {
        Self::ReportSummary { speed: self.speed }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReportSumSpeed<const N: usize> {
    pub speed: Stats,
}

impl<const N: usize> ReportSummary for ReportSumSpeed<N> {
    const N_FIELDS: usize = 4;
    type Fields = [(&'static str, Value); 4];
    fn fields(&self) -> [(&'static str, Value); 4] {
        [
            ("speed_mean", self.speed.mean.into()),
            ("speed_std", self.speed.std.into()),
            ("speed_min", self.speed.min.into()),
            ("speed_max", self.speed.max.into()),
        ]
    }
}

#[derive(Default)]
pub struct BenchmarkSpeed<const N: usize> {}

impl<const N: usize> Benchmark for BenchmarkSpeed<N> {
    const NAME: &'static str = "BenchmarkSpeed";
    type Report = ReportSpeed<N>;
    type ReportAccumulator = ReportAccSpeed<N>;
}

impl<const N: usize, R> RunBenchmark<R> for BenchmarkSpeed<N>
where
    R: RandomNumberGenerator,
{
    fn bench(&self, rng: &mut R) -> Self::Report {
        for _ in 0..100 {
            black_box(rng.next());
        }

        let t1 = Instant::now();
        for _ in 0..N {
            black_box(rng.next());
        }
        let t2 = Instant::now() - t1;

        Self::Report {
            t: t2,
            s: t2.as_nanos() / (N as u128),
        }
    }
}
