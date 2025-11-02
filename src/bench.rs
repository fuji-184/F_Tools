#[macro_export]
macro_rules! bench {
    ($iter:expr, $($body:tt)*) => {{
        use std::time::{Instant, Duration};
        let mut raws = [Duration::from_nanos(0); $iter];

        for i in 0..$iter {
            let start = Instant::now();
            $($body)*
            raws[i] = start.elapsed();
        }

        let mut samples = [0u64; $iter];
        std::thread::sleep(std::time::Duration::from_millis(500));

        for i in 0..$iter {
            samples[i] = raws[i].as_nanos() as u64;
        }

        samples.sort_unstable();
        let min = samples[0];
        let max = samples[$iter - 1];
        let total: u64 = samples.iter().sum();
        let avg = total / $iter;
        let p90 = samples[(($iter * 90 / 100) as usize).min($iter - 1)];
        let p99 = samples[(($iter * 99 / 100) as usize).min($iter - 1)];
        
        fn fmt_time_ns(t: u64) -> (&'static str, f64) {
            if t < 1_000 {
                ("ns", t as f64)
            } else if t < 1_000_000 {
                ("µs", t as f64 / 1_000.0)
            } else if t < 1_000_000_000 {
                ("ms", t as f64 / 1_000_000.0)
            } else {
                ("s", t as f64 / 1_000_000_000.0)
            }
        }

        let (umin, vmin) = fmt_time_ns(min);
        let (umax, vmax) = fmt_time_ns(max);
        let (uavg, vavg) = fmt_time_ns(avg);
        let (up90, vp90) = fmt_time_ns(p90);
        let (up99, vp99) = fmt_time_ns(p99);

        println!(
            "\n\ninfo: ns < µs < ms < s\nbench ({} iters):\nmin={:.2} {}\nmax={:.2} {}\navg={:.2} {}\np90={:.2} {}\np99={:.2} {}\n",
            $iter, vmin, umin, vmax, umax, vavg, uavg, vp90, up90, vp99, up99
        );
        

    }};
}
