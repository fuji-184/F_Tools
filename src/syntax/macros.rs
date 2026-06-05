
// Wraps a block of operations in a closure that returns Result, enabling `?` to propagate
// errors from multiple fallible calls without requiring a surrounding Result-returning function.
// Useful for capturing a group of operations and handling all errors in one place.
// Supports an explicit error type annotation via `as Result<_, ErrType>` when inference fails.
#[macro_export]
macro_rules! result {
    (as Result<_, $err_ty:ty> => $($tokens:tt)*) => {
        (|| -> Result<_, $err_ty> { Ok({ $($tokens)* }) })()
    };
    ($($tokens:tt)*) => {
        (|| -> Result<_, _> { Ok({ $($tokens)* }) })()
    };
}

// Ternary operator shorthand. Evaluates a boolean condition and returns one of two
// expressions. Avoids verbose if/else blocks for simple conditional value selection.
#[macro_export]
macro_rules! pick {
    ($c:expr, $t:expr, $e:expr) => {
        if $c { $t } else { $e }
    };
}

pub trait IntoOption {
    type Value;
    fn into_option(self) -> Option<Self::Value>;
}

impl<T> IntoOption for Option<T> {
    type Value = T;
    fn into_option(self) -> Option<T> {
        self
    }
}

impl<T, E> IntoOption for Result<T, E> {
    type Value = T;
    fn into_option(self) -> Option<T> {
        self.ok()
    }
}

// Optional chaining via `?.field` syntax, similar to JavaScript's `?.` operator.
// Traverses a chain of fields that return Option or Result, short-circuiting to None
// the moment any field in the chain is absent or failed. Eliminates nested match/if-let chains.
#[macro_export]
macro_rules! get {
    ($obj:ident) => {
        Some($obj)
    };
    ($obj:ident $(?. $prop:ident)+) => {
        Some($obj)
            $(.and_then(|o| $crate::IntoOption::into_option(o.$prop)))*
    };
}

// Unwraps an Option and binds the inner value, or early-returns from the enclosing
// function if the value is None. Accepts an optional return value; defaults to unit return.
// Replaces repetitive guard clauses like `let x = match opt { Some(v) => v, None => return }`.
#[macro_export]
macro_rules! unwrap_or_return {
    ($e:expr, $ret:expr) => {
        match $e {
            Some(v) => v,
            None => return $ret,
        }
    };
    ($e:expr) => {
        match $e {
            Some(v) => v,
            None => return,
        }
    };
}

// Evaluates a boolean condition and executes an else branch if it is false.
// The else branch can be a return with a value, a bare return, or any side-effect expression.
// Intended for guard-style checks where the happy path continues after the macro.
#[macro_export]
macro_rules! run {
    ($($tokens:tt)*) => {
        $crate::__run_internal!(() $($tokens)*);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __run_internal {
    (($($cond:tt)*) else return $ret:expr) => {
        let cond = { $($cond)* };
        if !cond {
            return $ret;
        }
    };

    (($($cond:tt)*) else return) => {
        let cond = { $($cond)* };
        if !cond {
            return;
        }
    };

    (($($cond:tt)*) else $fallback:expr) => {
        let cond = { $($cond)* };
        if !cond {
            $fallback;
        }
    };

    (($($head:tt)*) $next:tt $($tail:tt)*) => {
        $crate::__run_internal!(($($head)* $next) $($tail)*);
    };
}

// Wraps a block of operations in a closure returning Option, enabling `?` to short-circuit
// to None from any operation that returns None or a failed Result inside the block.
// Mirror of result! but for Option-returning contexts.
#[macro_export]
macro_rules! option {
    ($($tokens:tt)*) => {
        (|| -> std::option::Option<_> { std::option::Option::Some({ $($tokens)* }) })()
    };
}

// Tests whether a value matches any one of multiple patterns, returning a bool.
// Compiles down to a single match expression with OR-ed arms.
// Replaces verbose `matches!(x, A) || matches!(x, B)` chains.
#[macro_export]
macro_rules! match_any {
    ($val:expr => $($pattern:pat),+ $(,)?) => {
        match $val {
            $($pattern)|+ => true,
            _ => false,
        }
    };
}

// Returns true if at least one of the given collections or strings is empty.
// Evaluates using short-circuit OR, so subsequent collections are not checked
// once an empty one is found. Useful as a fast guard before processing multiple inputs.
#[macro_export]
macro_rules! any_empty {
    ($($collection:expr),+ $(,)?) => {
        $( $collection.is_empty() )||+
    };
}

pub trait IsSmartPointer {
    type Target: Clone;
    fn cheap_clone(&self) -> Self::Target;
}

impl<T: ?Sized> IsSmartPointer for std::rc::Rc<T> {
    type Target = std::rc::Rc<T>;
    fn cheap_clone(&self) -> Self::Target {
        self.clone()
    }
}

impl<T: ?Sized> IsSmartPointer for std::sync::Arc<T> {
    type Target = std::sync::Arc<T>;
    fn cheap_clone(&self) -> Self::Target {
        self.clone()
    }
}

// Clones one or more Rc or Arc variables into a closure scope before the closure is defined.
// Prevents move semantics from consuming the originals, allowing them to remain usable
// after the closure is created. Mirrors the common `let x = Arc::clone(&x)` pattern
// used before async blocks or thread spawns.
#[macro_export]
macro_rules! ref_clone {
    ([$($var:ident),+ $(,)?], $closure:expr) => {
        {
            $(let $var = $crate::IsSmartPointer::cheap_clone(&$var);)+
            $closure
        }
    };
}

// Derives an as_str() method on an enum that returns the variant name as a &'static str.
// Avoids implementing Display or Debug just to get a plain string name.
// Supports visibility modifiers, attributes, and explicit discriminant values.
#[macro_export]
macro_rules! enum_str {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($(#[$v_meta:meta])* $variant:ident $(= $val:expr)?),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($(#[$v_meta])* $variant $(= $val)?),*
        }

        impl $name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant)),*
                }
            }
        }
    };
}

// Wraps a sync or async function with entry/exit timing instrumentation using println.
// Records an Instant at function entry and prints elapsed duration on exit.
// Supports the full function signature including visibility, attributes, and return types.
// Intended for quick profiling during development without external tracing dependencies.
#[macro_export]
macro_rules! fn_log {
    (
        $(#[$meta:meta])*
        $vis:vis async fn $name:ident ($($arg:tt)*) $(-> $ret:ty)? $body:block
    ) => {
        $(#[$meta])*
        $vis async fn $name ($($arg)*) $(-> $ret)? {
            let _start = std::time::Instant::now();
            println!("[START] Executing async fn: {}", stringify!($name));
            let result = { $body };
            println!("[END] Finished fn: {} in {:?}", stringify!($name), _start.elapsed());
            result
        }
    };
    (
        $(#[$meta:meta])*
        $vis:vis fn $name:ident ($($arg:tt)*) $(-> $ret:ty)? $body:block
    ) => {
        $(#[$meta])*
        $vis fn $name ($($arg)*) $(-> $ret)? {
            let _start = std::time::Instant::now();
            println!("[START] Executing fn: {}", stringify!($name));
            let result = { $body };
            println!("[END] Finished fn: {} in {:?}", stringify!($name), _start.elapsed());
            result
        }
    };
}

// Derives a positional new() constructor for a struct whose parameters match the field
// declaration order. Eliminates boilerplate constructor implementations for simple data structs.
// Supports visibility modifiers and field-level attributes.
#[macro_export]
macro_rules! struct_new {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($f_vis:vis $field:ident : $ty:ty),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $($f_vis $field : $ty),*
        }

        impl $name {
            pub fn new($($field : $ty),*) -> Self {
                Self { $($field),* }
            }
        }
    };
}

// Defines a unit struct with associated typed constants representing individual bitmask flags.
// Each constant is a value of the specified primitive type (e.g. u8, u32) and can be
// combined with bitwise operators. Avoids external crates like bitflags for simple cases.
#[macro_export]
macro_rules! bit_flags {
    ($vis:vis struct $name:ident : $ty:ty { $($flag:ident = $val:expr),* $(,)? }) => {
        $vis struct $name;
        impl $name {
            $(pub const $flag: $ty = $val;)*
        }
    };
}

// Pattern-matches on a &dyn Any reference by concrete type, executing the first arm whose
// type matches. Falls through to a _ default block if no type matches.
// Useful for dispatching on heterogeneous collections or boxed trait objects without
// manually chaining downcast_ref calls.
#[macro_export]
macro_rules! downcast_ref {
    ($any_expr:expr, { $($arms:tt)* }) => {
        {
            let any = $any_expr;
            $crate::__select_branch_internal!(any, $($arms)*)
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __select_branch_internal {
    ($any:ident, $ty:ty => $var:ident $body:block, $($tail:tt)*) => {
        if let Some($var) = $any.downcast_ref::<$ty>() {
            $body
        } else {
            $crate::__select_branch_internal!($any, $($tail)*)
        }
    };

    ($any:ident, _ => $fallback:block) => {
        $fallback
    };

    ($any:ident $(,)?) => {
        {}
    };
}

// Prints the size and alignment in bytes for one or more types to stdout.
// Intended as a development utility for inspecting memory layout of structs,
// enums, or primitives — useful when optimizing padding or designing serializable formats.
#[macro_export]
macro_rules! size_of {
    ($($ty:ty),+ $(,)?) => {
        {
            println!("=== MEMORY LAYOUT INSPECTION ===");
            $(
                println!("{:<20} => Size: {} bytes, Align: {} bytes", 
                    stringify!($ty), 
                    std::mem::size_of::<$ty>(), 
                    std::mem::align_of::<$ty>()
                );
            )+
        }
    };
}

// Declares a lazily-initialized, RwLock-protected global variable wrapped in OnceLock.
// Exposes typed read(f) and write(f) accessors that accept closures, hiding lock
// acquisition and poison handling from call sites. Suitable for shared mutable state
// that must be accessible across threads without passing references explicitly.
#[macro_export]
macro_rules! global_mut {
    ($vis:vis $name:ident : $ty:ty = $init:expr) => {
        $vis struct $name;
        impl $name {
            fn registry() -> &'static std::sync::RwLock<$ty> {
                static HOLDER: std::sync::OnceLock<std::sync::RwLock<$ty>> = std::sync::OnceLock::new();
                HOLDER.get_or_init(|| std::sync::RwLock::new($init))
            }

            $vis fn read<R>(f: impl FnOnce(&$ty) -> R) -> R {
                let guard = match Self::registry().read() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                f(&guard)
            }

            $vis fn write<R>(f: impl FnOnce(&mut $ty) -> R) -> R {
                let mut guard = match Self::registry().write() {
                    Ok(g) => g,
                    Err(poisoned) => poisoned.into_inner(),
                };
                f(&mut guard)
            }
        }
    };
}

// Returns the first Some value from a list of Options, evaluating left to right.
// Behaves like a null-coalescing chain: if the first is None, tries the next, and so on.
// Returns None only if all options in the list are None.
#[macro_export]
macro_rules! either {
    ($expr:expr $(,)?) => {
        $expr
    };
    ($expr:expr, $($tail:expr),+ $(,)?) => {
        match $expr {
            std::option::Option::Some(val) => std::option::Option::Some(val),
            std::option::Option::None => $crate::either!($($tail),+),
        }
    };
}

// Unwraps an Option or Result inside a loop body, breaking out of the loop if the value
// is None or Err. Use the `res` variant for Result inputs.
// Eliminates explicit match blocks inside iteration loops that must stop on missing data.
#[macro_export]
macro_rules! unwrap_or_break {
    ($expr:expr) => {
        match $expr {
            std::option::Option::Some(val) => val,
            _ => break,
        }
    };
    ($res_expr:expr, res) => {
        match $res_expr {
            std::result::Result::Ok(val) => val,
            _ => break,
        }
    };
}

// Creates a named loop block that exposes a stop!(value) macro for breaking out with
// a return value. Allows loops to produce a value without requiring a separate mutable
// variable to capture the result. Both macros are functionally identical.
#[macro_export]
macro_rules! looping {
    ($($tokens:tt)*) => {
        'block: loop {
            macro_rules! stop {
                ($value:expr) => {
                    break 'block $value;
                };
            }
            $($tokens)*
        }
    };
}
#[macro_export]
macro_rules! while_loop {
    ($($tokens:tt)*) => {
        'block: loop {
            macro_rules! stop {
                ($value:expr) => {
                    break 'block $value;
                };
            }
            $($tokens)*
        }
    };
}

// Extracts multiple fields from an Option<Struct> into individual local bindings in one step.
// Executes a bail expression if the Option is None, preventing access to uninitialized bindings.
// Replaces verbose if-let + field extraction patterns when multiple fields are needed at once.
#[macro_export]
macro_rules! destructure {
    ($opt:expr => { $($field:ident),+ } else $bail:expr) => {
        let ($($field),+) = match $opt {
            std::option::Option::Some(obj) => ($(obj.$field),+),
            _ => { $bail; }
        };
    };
}

// Asserts that all listed boolean conditions are true, executing a bail expression if any fail.
// Conditions are AND-ed together; the bail runs on the first combined failure.
// Used as a multi-condition guard at the top of functions or blocks.
#[macro_export]
macro_rules! require {
    ($($cond:expr),+ ; else $bail:expr) => {
        if !($( $cond )&&+) {
            $bail;
        }
    };
}

// Functional alias of either!. Returns the first Some from a list of Options.
// Provided as a semantic alternative when the intent is value selection rather than fallback chaining.
#[macro_export]
macro_rules! either_val {
    ($expr:expr $(,)?) => {
        $expr
    };
    ($expr:expr, $($tail:expr),+ $(,)?) => {
        match $expr {
            std::option::Option::Some(val) => std::option::Option::Some(val),
            std::option::Option::None => $crate::either_val!($($tail),+),
        }
    };
}

// Unwraps a Result into a local binding on Ok, or executes a typed bail block on Err
// with the error value bound and accessible inside the block.
#[macro_export]
macro_rules! let_with_err {
    ($var:pat = $expr:expr, Err($err:ident) => $bail:block) => {
        let $var = match $expr {
            std::result::Result::Ok(val) => val,
            std::result::Result::Err($err) => { $bail }
        };
    };
}

// Unwraps multiple Options into individual local bindings in sequence.
// If any Option is None, the bail expression is executed immediately and remaining
// bindings are not evaluated. Useful for extracting several optional values
// at the start of a function with a single shared failure path.
#[macro_export]
macro_rules! unwrap_all {
    ($($var:ident = $expr:expr),+ ; else $bail:expr) => {
        $(
            let $var = match $expr {
                std::option::Option::Some(val) => val,
                std::option::Option::None => { $bail; }
            };
        )+
    };
}

// Threads a mutable value through a sequence of modifier functions, each receiving &mut T,
// and returns the final value. Models a builder-style mutation chain without requiring
// method chaining on the type itself. Each function in the chain mutates the value in place.
#[macro_export]
macro_rules! chain_call {
    ($target:expr => $func:ident) => {
        {
            let mut _obj = $target;
            $func(&mut _obj);
            _obj
        }
    };
    ($target:expr => $func:ident => $($tail:tt)+) => {
        {
            let mut _obj = $target;
            $func(&mut _obj);
            $crate::chain_call!(_obj => $($tail)+)
        }
    };
}

// Compile-time assertion that an expression matches a specific type.
// Fails at compile time if the type does not match, catching accidental type mismatches
// without runtime overhead. Useful for documenting and enforcing expected types inline.
#[macro_export]
macro_rules! type_assert {
    ($var:expr => $ty:ty) => {
        let _: $ty = $var;
    };
}

// Emits a compiler deprecation warning at compile time using a synthetic deprecated struct.
// Used to attach visible diagnostic messages to code paths, constants, or modules
// that are pending removal or require attention, without blocking compilation.
#[macro_export]
macro_rules! compile_note {
    ($msg:expr) => {
        const _: () = {
            #[deprecated(note = $msg)]
            struct CompileNote;
            fn trigger() { let _ = CompileNote; }
        };
    };
}

ftest::test!(macro_tests, {
    use std::sync::Arc;
    use std::rc::Rc;
    use std::any::Any;

    result_basic_ok {
        let r: Result<i32, Box<dyn std::error::Error>> = result!(1 + 1);
        assert!(r.is_ok());
        assert_eq!(r.unwrap(), 2);
    }

    result_explicit_err_type {
        let r = result!(as Result<_, String> => "hello".to_string());
        assert_eq!(r.unwrap(), "hello");
    }

    result_propagates_question_mark {
        let inner: Result<i32, &str> = Err("oops");
        let r: Result<i32, &str> = result!(inner?);
        assert!(r.is_err());
    }

    pick_true_branch {
        let v = pick!(true, 10, 20);
        assert_eq!(v, 10);
    }

    pick_false_branch {
        let v = pick!(false, 10, 20);
        assert_eq!(v, 20);
    }

    pick_expression_condition {
        let x = 5;
        let v = pick!(x > 3, "big", "small");
        assert_eq!(v, "big");
    }

    get_bare_ident {
        let x = 42;
        let r = get!(x);
        assert_eq!(r, Some(42));
    }

    get_chained_some {
        struct A { b: Option<i32> }
        let a = A { b: Some(99) };
        let r = get!(a ?.b);
        assert_eq!(r, Some(99));
    }

    get_chained_none_short_circuits {
        struct A { b: Option<i32> }
        let a = A { b: None };
        let r = get!(a ?.b);
        assert_eq!(r, None);
    }

    unwrap_or_return_some {
        fn f() -> i32 {
            let v = unwrap_or_return!(Some(7), 0);
            v * 2
        }
        assert_eq!(f(), 14);
    }

    unwrap_or_return_none_with_fallback {
        fn f() -> i32 {
            let _v = unwrap_or_return!(None::<i32>, -1);
            999
        }
        assert_eq!(f(), -1);
    }

    unwrap_or_return_none_unit {
        let mut called = false;
        let mut run = || {
            let _v = unwrap_or_return!(None::<i32>);
            called = true;
        };
        run();
        assert!(!called);
    }

    option_returns_some {
        let r = option!(1 + 1);
        assert_eq!(r, Some(2));
    }

    option_propagates_none {
        let inner: Option<i32> = None;
        let r = option!(inner?);
        assert_eq!(r, None);
    }

    match_any_found {
        let x = 3u32;
        assert!(match_any!(x => 1, 2, 3));
    }

    match_any_not_found {
        let x = 5u32;
        assert!(!match_any!(x => 1, 2, 3));
    }

    any_empty_one_empty {
        let a = vec![1, 2];
        let b: Vec<i32> = vec![];
        assert!(any_empty!(a, b));
    }

    any_empty_all_nonempty {
        let a = vec![1];
        let b = vec![2];
        assert!(!any_empty!(a, b));
    }

    ref_clone_arc {
        let data = Arc::new(vec![1, 2, 3]);
        let closure = ref_clone!([data], move || data.len());
        assert_eq!(closure(), 3);
    }

    ref_clone_rc {
        let data = Rc::new(42);
        let closure = ref_clone!([data], move || *data * 2);
        assert_eq!(closure(), 84);
    }

    enum_str_as_str {
        enum_str! { enum Color { Red, Green, Blue } }
        assert_eq!(Color::Red.as_str(), "Red");
        assert_eq!(Color::Green.as_str(), "Green");
        assert_eq!(Color::Blue.as_str(), "Blue");
    }

    struct_new_creates_correct_struct {
        struct_new! {
            struct Point { pub x: i32, pub y: i32 }
        }
        let p = Point::new(3, 4);
        assert_eq!(p.x, 3);
        assert_eq!(p.y, 4);
    }

    bit_flags_constants {
        bit_flags! {
            pub struct Perms: u32 { READ = 0b001, WRITE = 0b010, EXEC = 0b100 }
        }
        assert_eq!(Perms::READ, 0b001);
        assert_eq!(Perms::WRITE, 0b010);
        assert_eq!(Perms::EXEC, 0b100);
    }

    bit_flags_bitwise_combine {
        bit_flags! { pub struct Flags: u8 { A = 1, B = 2 } }
        assert_eq!(Flags::A | Flags::B, 3);
    }

    downcast_ref_matches_type {
        let val: Box<dyn Any> = Box::new(42i32);
        let mut matched = false;
        downcast_ref!(val.as_ref(), {
            i32 => v { matched = *v == 42; },
            _ => {}
        });
        assert!(matched);
    }

    downcast_ref_uses_default {
        let val: Box<dyn Any> = Box::new("hello");
        let mut used_default = false;
        downcast_ref!(val.as_ref(), {
            i32 => _v {},
            _ => { used_default = true; }
        });
        assert!(used_default);
    }

    global_mut_read_initial {
        global_mut!(pub COUNTER: i32 = 0);
        let v = COUNTER::read(|c| *c);
        assert_eq!(v, 0);
    }

    global_mut_write_then_read {
        global_mut!(pub SCORE: i32 = 0);
        SCORE::write(|s| *s = 42);
        assert_eq!(SCORE::read(|s| *s), 42);
    }

    either_first_some {
        assert_eq!(either!(Some(1), Some(2)), Some(1));
    }

    either_skips_none {
        assert_eq!(either!(None::<i32>, Some(2)), Some(2));
    }

    either_all_none {
        assert_eq!(either!(None::<i32>, None::<i32>), None);
    }

    unwrap_or_break_on_none {
        let opts = vec![Some(1), Some(2), None, Some(4)];
        let mut collected = vec![];
        for opt in &opts {
            let v = unwrap_or_break!(*opt);
            collected.push(v);
        }
        assert_eq!(collected, vec![1, 2]);
    }

    unwrap_or_break_on_err {
        let results: Vec<Result<i32, &str>> = vec![Ok(1), Ok(2), Err("x"), Ok(4)];
        let mut collected = vec![];
        for r in &results {
            let v = unwrap_or_break!(r.clone(), res);
            collected.push(v);
        }
        assert_eq!(collected, vec![1, 2]);
    }

    looping_stop_with_value {
        let result = looping! {
            let x = 10;
            stop!(x * 2);
        };
        assert_eq!(result, 20);
    }

    require_passes_when_true {
        let mut reached = false;
        let mut run = || {
            require!(1 == 1, 2 == 2 ; else return);
            reached = true;
        };
        run();
        assert!(reached);
    }

    require_bails_when_false {
        let mut reached = false;
        let mut run = || {
            require!(1 == 2 ; else return);
            reached = true;
        };
        run();
        assert!(!reached);
    }

    unwrap_all_all_some {
        let run = || -> i32 {
            unwrap_all!(a = Some(1), b = Some(2) ; else return -1);
            a + b
        };
        assert_eq!(run(), 3);
    }

    unwrap_all_bail_on_none {
        let run = || -> i32 {
            unwrap_all!(a = Some(1), b = None::<i32> ; else return -1);
            a + b
        };
        assert_eq!(run(), -1);
    }

    let_with_err_ok {
        let run = || -> i32 {
            let_with_err!(v = Ok::<i32, &str>(42), Err(_e) => { return -1 });
            v
        };
        assert_eq!(run(), 42);
    }

    let_with_err_bails_on_err {
        let run = || -> i32 {
            let_with_err!(v = Err::<i32, &str>("bad"), Err(_e) => { return -99 });
            v
        };
        assert_eq!(run(), -99);
    }

    fn_log_returns_value {
        fn_log! {
            fn add(a: i32, b: i32) -> i32 { a + b }
        }
        assert_eq!(add(3, 4), 7);
    }

    either_val_first_some {
        assert_eq!(either_val!(Some(1), Some(2)), Some(1));
    }

    either_val_skips_none {
        assert_eq!(either_val!(None::<i32>, Some(9)), Some(9));
    }

    run_else_return_on_false {
        fn check(v: i32) -> i32 {
            run!(v > 10 else return -1);
            1
        }
        assert_eq!(check(5), -1);
        assert_eq!(check(20), 1);
    }

    run_else_fallback_on_false {
        let mut side_effect = false;
        run!(1 == 2 else { side_effect = true; });
        assert!(side_effect);
    }

    type_assert_compiles {
        let x: i32 = 5;
        type_assert!(x => i32);
    }
});