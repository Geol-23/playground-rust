use datastructs::OrderedHashMap;

use crate::core::{Seed, SeedableRNG};

// Traits -----------------------------------------------------------------------------------------
pub trait Report {}

pub trait ReportAccumulator {
    type Report: Report;
    type ReportSummary: ReportSummary;

    fn new() -> Self;
    fn mix(&mut self, other: &Self::Report);
    fn summary(&self) -> Self::ReportSummary;
}

#[derive(Debug, Clone, Copy)]
pub enum Value {
    UInt(u64),
    Float(f64),
    Size(usize),
}

impl Value {
    pub fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Value::UInt(a), Value::UInt(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => a.total_cmp(b),
            (Value::Size(a), Value::Size(b)) => a.cmp(b),
            (_, _) => panic!("cmp between two different variants"),
        }
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Value::UInt(value)
    }
}
impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value)
    }
}
impl From<usize> for Value {
    fn from(value: usize) -> Self {
        Value::Size(value)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(x) => write!(f, "{x:.3e}"),
            Value::Size(x) => write!(f, "{x}"),
            Value::UInt(x) => write!(f, "{x}"),
        }
    }
}

type ReportSummaryField = (&'static str, Value);
pub trait ReportSummary {
    const N_FIELDS: usize;
    type Fields: AsRef<[ReportSummaryField]>
        + AsMut<[ReportSummaryField]>
        + IntoIterator<Item = ReportSummaryField>;
    fn fields(&self) -> Self::Fields;
}

pub trait Benchmark: Default {
    const NAME: &'static str;
    type Report: Report;
    type ReportAccumulator: ReportAccumulator<Report = Self::Report>;
}

pub trait RunBenchmark<R>: Benchmark {
    fn bench(&self, rng: &mut R) -> Self::Report;
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub rng: &'static str,
    pub benchmark: &'static str,
    pub fields: OrderedHashMap<&'static str, Value>,
}

pub fn run<R, B>(seeds: impl Iterator<Item = u64>) -> BenchmarkResult
where
    R: SeedableRNG,
    B: Benchmark + RunBenchmark<R>,
{
    let benchmark = B::default();
    println!(
        "running {} against {}...",
        B::NAME,
        std::any::type_name::<R>().split("::").last().unwrap()
    );
    let summmary = seeds
        .map(|s| {
            let mut rng = R::new(Seed::new(s));
            benchmark.bench(&mut rng)
        })
        .fold(B::ReportAccumulator::new(), |mut a, i| {
            a.mix(&i);
            a
        })
        .summary();
    BenchmarkResult {
        rng: std::any::type_name::<R>(),
        benchmark: B::NAME,
        fields: OrderedHashMap::from_iter(summmary.fields()),
    }
}

#[macro_export]
macro_rules! make_suite {
    (
        benchmarks: [$( $bm:ty ),+ $(,)?],
        seeds: $bg:tt,
        rngs: $rngs:tt,
    ) => {{

        use $crate::{
            testkit::{
                bitgen::{Sampler, SamplerTrait},
                framework::{Benchmark, ReportAccumulator, RunBenchmark, run}
            },
            core::{RandomNumberGenerator, SeedableRNG, Seed},
        };

        [$( $crate::make_suite!( @row $bg; $bm; $rngs; ) ),+]
    }};

    (@row $bg:tt; $bm:ty; [$( $rng:ty ),+ $(,)?];) => {
        [$( run::<$rng, $bm>( $crate::make_suite!(@seed $bg) ) ),+]
    };


    (@seed [$bgname:ty {$start:expr; $n:expr; $stop:expr}, $($tail:tt)+]) => {
        Sampler::<{ $n }, { $start }, { $stop }>::apply::<$bgname>()
            .chain(
                $crate::make_suite!(@seed [$($tail)+])
            )
    };

    (@seed [$bgname:ty {$start:expr; $n:expr; $stop:expr} $(,)?]) => {
        Sampler::<{ $n }, { $start }, { $stop }>::apply::<$bgname>()
    };
}

pub fn print_benchmark(
    data: &[BenchmarkResult],
    cols: &[&'static str],
) -> Result<(), &'static str> {
    let first = data.first().ok_or("empty data")?;
    let fields: Vec<_> = first.fields.keys().map(|n| *n).collect();
    if !data.iter().all(|r| r.benchmark == first.benchmark) {
        return Err("heterogeneous benchmarks detected");
    }

    let null_cols = cols
        .iter()
        .filter_map(|c| (!fields.contains(c)).then_some(c))
        .count();
    if null_cols != 0 {
        return Err("non-existient cols found");
    }

    let mut cols: Vec<_> = cols.into_iter().map(|s| *s).collect();
    if cols.is_empty() {
        cols = fields;
    }

    let mut table: Vec<Vec<String>> = Vec::new();

    // Title Row
    table.push(
        std::iter::once(first.benchmark.to_string())
            .chain(cols.iter().map(|n| n.to_string()))
            .collect::<Vec<_>>(),
    );

    for row in data {
        let rng = row.rng.split("::").last().unwrap();
        table.push(
            std::iter::once(rng.to_string())
                .chain(
                    row.fields
                        .iter()
                        .filter_map(|(n, v)| cols.contains(n).then_some(format!("{v}"))),
                )
                .collect::<Vec<_>>(),
        )
    }

    let mut widths: Vec<usize> = table.first().unwrap().iter().map(|s| s.len()).collect();
    for row in &table[1..] {
        for (r, w) in row.iter().zip(widths.iter_mut()) {
            *w = *w.max(&mut r.len()) + 1;
        }
    }

    for (i, row) in table.iter().enumerate() {
        if i == 1 {
            print!("\n");
        }
        for (r, w) in row.iter().zip(widths.iter()) {
            print!("{r:>w$}");
        }
        print!("\n")
    }

    Ok(())
}

// Stats ------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub n: usize,
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub max: f64,
}
impl Stats {
    pub fn empty() -> Self {
        Self {
            n: 0,
            mean: 0.0,
            std: 0.0,
            min: f64::MAX,
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
    T: Copy + std::cmp::Ord,
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
