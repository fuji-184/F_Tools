
/*
Lightweight task scheduler for executing periodic background operations based on time intervals.

This scheduler maintains a registry of named tasks, each associated with a specific execution 
cadence. It continuously monitors the elapsed time since each job's last invocation, triggering 
execution only when the predefined duration has expired. It is primarily used for automating 
repetitive system maintenance—such as periodic cache flushing, automated telemetry cleanup, 
or status heartbeat reporting—providing an internal mechanism to batch and track the success 
rates of repetitive background routines without requiring heavy OS-level process scheduling.
*/

use std::time::{Duration, Instant};

pub struct CronJob<'a> {
    pub name: &'a str,
    pub interval: Duration,
    pub last_run: Instant,
    pub action: Box<dyn FnMut() -> Result<(), String> + 'a>,
    pub run_count: u64,
    pub success_count: u64,
}

pub struct CronScheduler<'a> {
    jobs: Vec<CronJob<'a>>,
}

impl<'a> CronScheduler<'a> {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn add_job<F>(&mut self, name: &'a str, interval: Duration, action: F)
    where
        F: FnMut() -> Result<(), String> + 'a
    {
        self.jobs.push(CronJob {
            name,
            interval,
            last_run: Instant::now(),
            action: Box::new(action),
            run_count: 0,
            success_count: 0,
        });
    }

    pub fn tick(&mut self) {
        for job in &mut self.jobs {
            if job.last_run.elapsed() >= job.interval {
                job.run_count += 1;
                if (job.action)().is_ok() {
                    job.success_count += 1;
                }
                job.last_run = Instant::now();
            }
        }
    }

    pub fn run_loop(&mut self, precision: Duration) {
        loop {
            self.tick();
            std::thread::sleep(precision);
        }
    }
    
    pub fn remove_job(&mut self, name: &str) -> bool {
        let original_len = self.jobs.len();
        self.jobs.retain(|j| j.name != name);
        self.jobs.len() < original_len
    }
    
    pub fn get_stats(&self, name: &str) -> Option<(u64, u64)> {
        self.jobs.iter()
            .find(|j| j.name == name)
            .map(|j| (j.run_count, j.success_count))
    }
}

ftest::test!(cron_scheduler_tests, {
    test_add_and_tick_trigger {
        let executed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let executed_clone = executed.clone();

        let mut scheduler = CronScheduler::new();
        scheduler.add_job("test_job", Duration::from_millis(10), move || {
            executed_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });

        std::thread::sleep(Duration::from_millis(15));
        scheduler.tick();

        assert!(executed.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(scheduler.get_stats("test_job"), Some((1, 1)));
    }

    test_tick_not_triggered_before_interval {
        let executed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let executed_clone = executed.clone();

        let mut scheduler = CronScheduler::new();
        scheduler.add_job("test_job", Duration::from_secs(10), move || {
            executed_clone.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        });

        scheduler.tick();

        assert!(!executed.load(std::sync::atomic::Ordering::SeqCst));
        assert_eq!(scheduler.get_stats("test_job"), Some((0, 0)));
    }

    test_remove_job {
        let mut scheduler = CronScheduler::new();

        scheduler.add_job("job1", Duration::from_secs(1), || Ok(()));
        scheduler.add_job("job2", Duration::from_secs(1), || Ok(()));

        assert!(scheduler.remove_job("job1"));
        assert!(!scheduler.remove_job("job1"));
        assert_eq!(scheduler.get_stats("job1"), None);
        assert!(scheduler.get_stats("job2").is_some());
    }

    test_job_failure_stats {
        let mut scheduler = CronScheduler::new();

        scheduler.add_job("fail_job", Duration::from_millis(10), || {
            Err("failed".to_string())
        });

        std::thread::sleep(Duration::from_millis(15));
        scheduler.tick();

        assert_eq!(scheduler.get_stats("fail_job"), Some((1, 0)));
    }
});