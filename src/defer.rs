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
