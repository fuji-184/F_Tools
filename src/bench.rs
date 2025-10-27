#[macro_export]
macro_rules! bench {
    ($iter:expr, $($body:tt)*) => {
        use std::time::Instant;

        let mut min = u128::MAX;
        let mut max = 0u128;
        let mut total = 0u128;

        let mut samples = Vec::with_capacity($iter);

        for _ in 0..$iter {
            let start = Instant::now();
            $($body)*
            let elapsed = start.elapsed().as_nanos();

            if elapsed < min { min = elapsed; }
            if elapsed > max { max = elapsed; }
            total += elapsed;
            samples.push(elapsed);
        }

        samples.sort_unstable();

        let avg = total as f64 / $iter as f64;
        let p90 = samples[(($iter as f64 * 0.9) as usize).min($iter - 1)];
        let p99 = samples[(($iter as f64 * 0.99) as usize).min($iter - 1)];

        println!(
            "bench ({} iters): min={} ns | max={} ns | avg={:.2} ns | p90={} ns | p99={} ns",
            $iter, min, max, avg, p90, p99
        );
    };
}
