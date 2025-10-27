#[cfg(debug_assertions)]
#[macro_export]
macro_rules! dlog {
    ($($arg:tt)*) => {
        println!($($arg)*);
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! dlog {
    ($($arg:tt)*) => {};
}
