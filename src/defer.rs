
/*
Scope-bound cleanup mechanism to automate resource management and guarantee execution.

This utility schedules a closure to be executed automatically when the current scope 
exits, regardless of whether the execution path finishes normally or terminates due 
to an error (panic or early return). It is primarily used to ensure critical cleanup 
operations—such as releasing hardware locks, closing file descriptors, resetting 
global state counters, or logging telemetry markers—by binding these tasks to the 
stack-based lifecycle of an RAII guard rather than manually tracking exit points.
*/

pub struct Defer<F: FnOnce()> {
    val: Option<F>,
}

impl<F: FnOnce()> Defer<F> {
    #[inline(always)]
    pub fn new(f: F) -> Self {
        Self { val: Some(f) }
    }
}

impl<F: FnOnce()> Drop for Defer<F> {
    #[inline(always)]
    fn drop(&mut self) {
        if let Some(val) = self.val.take() {
            val();
        }
    }
}

#[macro_export]
macro_rules! defer {
    ($($body: tt)*) => {
        let _defer = crate::defer::Defer::new(|| { $($body)* });
    };
}

