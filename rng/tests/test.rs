/*
 * Shorter-than-expected periods for some seed states (such seed states may be called "weak" in this context);
 * Lack of uniformity of distribution for large quantities of generated numbers;
 * Correlation of successive values;
 * Poor dimensional distribution of the output sequence;
 * Distances between where certain values occur are distributed differently from those in a random sequence distribution.
 */
use rng::generators;
use rng::testlib::{Bound, RNGBiasTest, RNGPeriodTest, SeedGen, StatBenchmark};

const CYCLE_LINEAR_N_SEEDS: u64 = 10_000;
const MAX_CYCLES: u32 = 500;
const BIAS_LINEAR_N_SEEDS: u64 = 1_000;
const BIAS_SAMPLES: u64 = 10_000;

macro_rules! create_test {
    ( $( $name:ident: $type:ty $(,)? ),+ ) => {$(
        mod $name {
            use super::{
                BENCH_LAM,
                BENCH_MU,
                MAX_CYCLES,
                CYCLE_LINEAR_N_SEEDS,
                BIAS_LINEAR_N_SEEDS,
                BIAS_SAMPLES,
                RNGBiasTest,
                RNGPeriodTest,
                SeedGen,
                $type
            };

            #[test]
            fn cycle_linear_seed() {
                let sampler = SeedGen::Sampler::new(CYCLE_LINEAR_N_SEEDS, 0, u64::MAX);
                let result =
                    <$type>::benchmark_period(sampler.apply(SeedGen::Linear::UNIT), MAX_CYCLES);
                result.bench(stringify!($type), BENCH_MU, BENCH_LAM);
            }

            #[test]
            fn bias_linear_seed() {
                let sampler = SeedGen::Sampler::new(BIAS_LINEAR_N_SEEDS, 0, u64::MAX);
                let result =
                    <$type>::benchmark_bias(sampler.apply(SeedGen::Linear::UNIT), BIAS_SAMPLES);
                panic!("{:#?}", result)
            }
        }
    )*};
}

#[cfg(test)]
mod tests {
    use super::*;
    use generators::*;

    const BENCH_MU: StatBenchmark = StatBenchmark {
        mean: Some(Bound::GE((MAX_CYCLES / 100 * 95) as f64)),
        std: Some(Bound::LE(1000 as f64)),
        min: Some(Bound::GE((MAX_CYCLES / 100 * 90) as f64)),
        max: None,
    };
    const BENCH_LAM: StatBenchmark = StatBenchmark {
        mean: Some(Bound::GE((MAX_CYCLES / 100 * 95) as f64)),
        std: Some(Bound::LE(1000 as f64)),
        min: Some(Bound::GE((MAX_CYCLES / 100 * 90) as f64)),
        max: None,
    };

    create_test!(
        test_middle_square: MiddleSquare,
    );
}
