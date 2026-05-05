use std::time::{Duration, Instant};

pub struct CronJob<'a> {
    pub name: &'static str,
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

    pub fn add_job<F>(&mut self, name: &'static str, interval: Duration, action: F)
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