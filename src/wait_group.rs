use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicU32};

#[cfg(feature = "libc")]
use crate::Futex;

pub struct WaitGroup {
    inner: Arc<WaitGroupInner>,
}

struct WaitGroupInner {
    count: AtomicUsize,
    mutex: Mutex<bool>, 
    cond: Condvar,
}

impl WaitGroup {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(WaitGroupInner {
                count: AtomicUsize::new(0),
                mutex: Mutex::new(false),
                cond: Condvar::new(),
            })
        }
    }

    pub fn add(&self, delta: usize) {
        self.inner.count.fetch_add(delta, Ordering::SeqCst);
    }

    pub fn done(&self) {
        let prev = self.inner.count.fetch_sub(1, Ordering::SeqCst);
        
        if prev == 1 {
            let mut is_done = self.inner.mutex.lock().unwrap();
            *is_done = true; 
            self.inner.cond.notify_all();
        }
    }

    pub fn wait(&self) {
        if self.inner.count.load(Ordering::SeqCst) == 0 {
            return;
        }

        let mut is_done = self.inner.mutex.lock().unwrap();

        while self.inner.count.load(Ordering::SeqCst) > 0 {
            is_done = self.inner.cond.wait(is_done).unwrap();
        }
    }
}

impl Clone for WaitGroup {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}


#[cfg(feature = "libc")]
pub struct FutexWaitGroup {
    inner: Arc<AtomicU32>,
}

#[cfg(feature = "libc")]
impl FutexWaitGroup {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn add(&self, delta: u32) {
        self.inner.fetch_add(delta, Ordering::SeqCst);
    }

    pub fn done(&self) {
        let prev = self.inner.fetch_sub(1, Ordering::SeqCst);
        
        if prev == 1 {
            Futex::wake(&self.inner, i32::MAX);
        }
    }

    pub fn wait(&self) {
        loop {
            let curr = self.inner.load(Ordering::SeqCst);
            
            if curr == 0 {
                break;
            }

            Futex::wait(&self.inner, curr);
        }
    }
}

#[cfg(feature = "libc")]
impl Clone for FutexWaitGroup {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

ftest::test!(wait_group_tests, {
    test_wait_group_no_tasks {
        let wg = WaitGroup::new();
        wg.wait();
    }

    test_wait_group_concurrent_execution.tokio {
        let wg = WaitGroup::new();
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..5 {
            wg.add(1);
            let wg_clone = wg.clone();
            let counter_clone = Arc::clone(&counter);
            
            tokio::spawn(async move {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                wg_clone.done();
            });
        }

        let wg_res = wg.clone();
        tokio::task::spawn_blocking(move || {
            wg_res.wait();
        })
        .await
        .unwrap();
        
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }
});

#[cfg(feature = "libc")]
ftest::test!(futex_wait_group_tests, {
    test_futex_wait_group_no_tasks {
        let wg = FutexWaitGroup::new();
        wg.wait();
    }

    test_futex_wait_group_concurrent_execution {
        let wg = FutexWaitGroup::new();
        let counter = Arc::new(AtomicUsize::new(0));
        let mut handles = vec![];

        for _ in 0..5 {
            wg.add(1);
            let wg_clone = wg.clone();
            let counter_clone = Arc::clone(&counter);

            handles.push(std::thread::spawn(move || {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                wg_clone.done();
            }));
        }

        wg.wait();
        assert_eq!(counter.load(Ordering::SeqCst), 5);

        for handle in handles {
            let _ = handle.join();
        }
    }
});