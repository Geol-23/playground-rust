/*
 * Shorter-than-expected periods for some seed states (such seed states may be called "weak" in this context);
 * Lack of uniformity of distribution for large quantities of generated numbers;
 * Correlation of successive values;
 * Poor dimensional distribution of the output sequence;
 * Distances between where certain values occur are distributed differently from those in a random sequence distribution.
 */

use std::cmp::Ord;

use rng::core::Seed;

#[derive(Clone, Copy, Debug)]
struct Stats {
    mean: f64,
    std: f64,
    min: f64,
    max: f64,
}
impl Stats {
    fn empty() -> Self {
        Self {
            mean: 0.0,
            std: 0.0,
            min: 0.0,
            max: 0.0,
        }
    }
}
impl<T> From<&[T]> for Stats
where
    T: Copy + Ord,
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
            mean,
            min,
            max,
            std: variance.sqrt(),
        }
    }
}

fn resolve_seed_period<S>(seeds: S, gen_limit: usize) -> Stats
where
    S: Iterator<Item = Seed>,
{
    todo!()
}
