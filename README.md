# F_Tools

F\_Tools is a comprehensive Rust utility toolkit library. It offers a wide range of tools fon concurrency, low level hardware optimizations, asynchronous patterns, and expressive syntax sugar to streamline development.

## Installation

Add the following to `Cargo.toml`:

```toml
[dependencies]
ftool = { git = "https://github.com/fuji-184/F_Tools.git" }
```

### Cargo Features

This crate exposes several features to enable optional functionalities:

-   `async_cross_thread`: Enables Tokio-based asynchronous utilities (`AsyncRetry`, `EventBus`, `AsyncTaskPool`, etc.).
-   `libc`: Enables utilities that depend on the `libc` crate for low level OS interactions (`MemMap`, `FutexWait`, `ThreadAffinity`, `ZeroCopy`, etc.).
-   `dev`: A convenient feature that enables both `async_cross_thread` and `libc`.

To enable a feature, add it to the dependency definition:

```toml
[dependencies]
ftool = { git = "https://github.com/fuji-184/F_Tools.git", features = ["async_cross_thread", "libc"] }
```

## Tools Overview

### Concurrency & Synchronization

-   **Sharded Data Structures** (`sharded`): A collection of thread-safe data structures (`ShardedMap`, `ShardedSet`, `ShardedVec`) that partition data across multiple shards to reduce lock contention.
-   **`AsyncBackpressure`**: A semaphore based mechanism to control and throttle the concurrency of asynchronous tasks.
-   **`AsyncLeakyBucket`**: An asynchronous rate limiter using the leaky bucket algorithm for smooth traffic shaping.
-   **`CircuitBreaker`**: A fault tolerance mechanism to prevent cascading failures by temporarily blocking calls to failing services.
-   **`WaitGroup` & `FutexWaitGroup`**: Synchronization primitives to block a thread until a collection of concurrent tasks completes. `FutexWaitGroup` is a high performance Linux-specific version.
-   **`FutexThreadPause`**: A low-level thread barrier using Linux futexes to synchronize multiple threads at a specific execution point.

### Asynchronous Utilities

-   **`AsyncDebounce`**: Delays execution of an action until a burst of events has ceased, processing only the last item.
-   **`AsyncGracefulShutdown`**: Coordinates a clean shutdown process, allowing cleanup logic to run when the application receives a termination signal.
-   **`AsyncOnce`**: A thread safe cell for performing asynchronous one time initialization.
-   **`AsyncRetry`**: A utility to automatically retry a failing asynchronous operation with optional exponential backoff.
-   **`AsyncTaskPool`**: A bounded asynchronous task pool with configurable workers, queue size, and graceful shutdown.
-   **`GlobalAsyncScope` & `LocalAsyncScope`**: Scoped task runners for structured concurrency, ensuring spawned tasks do not outlive their scope.
-   **`EventBus`**: A broadcast channel for many-to-many, decoupled communication between components.
-   **`Notif`**: A single value `watch` channel that notifies multiple subscribers of changes.

### Memory & Performance

-   **`MemMap`**: High performance, memory mapped file I/O that allows treating file contents as a memory slice for zero copy access.
-   **`PersistentVmm`**: A persistent Virtual Memory Manager built on `MemMap` that provides a lock free bump allocator for durable, on disk data structures.
-   **`InterProcessMemory`**: Manages shared memory for ultra-low-latency data exchange between separate processes.
-   **`ZeroCopy`**: A utility for kernel level, zero copy data transfer between file descriptors using `splice` and `sendfile`.
-   **`SimdCopy`**: High bandwidth memory copy accelerated with SIMD instructions (AVX, SSE, NEON).
-   **`SimdScanner`**: SIMD accelerated byte scanning for high throughput pattern matching in memory.
-   **`CycleCounter`**: A high precision profiler that reads CPU cycle counts for benchmarking.
-   **`ThreadAffinity`**: Provides control over thread scheduling, including pinning threads to specific CPU cores.
-   **`NumaMemoryBind`**: Manages memory allocation on NUMA systems to optimize data locality.

### Data Structures & Utilities

-   **`AtomicBitSet` & `BitSet`**: Space efficient sets for managing boolean flags using bitwise operations. `AtomicBitSet` is thread safe.
-   **`BloomFilter`**: A probabilistic data structure to test for element membership with zero false negatives.
-   **`TtlKvCache`**: A thread safe, in memory key value cache where entries expire after a specified Time To Live (TTL).
-   **`AtomicTimeSeriesId`**: A thread safe, Snowflake-inspired unique ID generator.
-   **`BitMaskPermissions`**: A utility for managing permissions and privileges using bitmasks.
-   **`BitPacker`**: A zero cost utility for packing smaller integer types into a larger one.
-   **`Memoize` & `MemoizeFile`**: A suite of memoization helpers for caching function or file-loading results, with variants for different threading and async contexts.
-   **`StateEventAction`**: A macro based utility for creating simple and deterministic Finite State Machines (FSMs).
-   **`SelfRef`**: A utility with macros to safely create self referential structs.

## Syntax Sugar

-   **`pick!`**: A ternary operator for concise conditional expressions.
    ```rust
    let status = pick!(is_ready, "Ready", "Waiting");
    ```

-   **`defer!`**: Ensures a block of code is executed when the current scope exits.
    ```rust
    defer! { println!("Cleaning up resources..."); }
    ```

-   **`result!`**: Wraps a block of code in a `Result`, allowing the use of the `?` operator.
    ```rust
    let data: Result<_, std::io::Error> = result! {
        let file = std::fs::File::open("data.txt")?;
        // ... more operations
        file
    };
    ```

-   **`enum_str!`**: Automatically derives an `as_str()` method for an enum.
    ```rust
    enum_str! {
        pub enum Status { Active, Inactive, Pending }
    }
    assert_eq!(Status::Active.as_str(), "Active");
    ```

-   **`ref_clone!`**: Clones `Arc` or `Rc` variables for use in a closure.
    ```rust
    let data = std::sync::Arc::new(vec![1, 2, 3]);
    let closure = ref_clone!([data], move || data.len());
    assert_eq!(closure(), 3);
    ```

-   **`global_mut!`**: Creates a lazily-initialized, thread-safe global variable.
    ```rust
    global_mut!(pub COUNTER: i32 = 0);
    
    COUNTER::write(|c| *c += 1);
    assert_eq!(COUNTER::read(|c| *c), 1);
    ```

-   **`get!`**: Optional chaining via `?.field` syntax, short circuiting to `None` on absent fields.
    ```rust
    struct User { address: Option<Address> }
    struct Address { city: Option<String> }
    
    let city = get!(user ?.address ?.city);
    ```

-   **`option!`**: Wraps a block in an `Option`, allowing `?` to short circuit to `None`.
    ```rust
    let result = option! {
        let x = some_option?;
        let y = another_option?;
        x + y
    };
    ```

-   **`unwrap_or_return!`**: Unwraps an `Option` or early returns from the enclosing function if `None`.
    ```rust
    fn process(data: Option<Vec<u8>>) -> String {
        let bytes = unwrap_or_return!(data, String::new());
        String::from_utf8_lossy(&bytes).into_owned()
    }
    ```

-   **`unwrap_or_break!`**: Unwraps an `Option` or `Result` inside a loop, breaking on absent value.
    ```rust
    for item in &items {
        let value = unwrap_or_break!(*item);
        process(value);
    }
    ```

-   **`unwrap_all!`**: Unwraps multiple `Option`s into bindings, bailing on the first `None`.
    ```rust
    fn render(state: &State) -> i32 {
        unwrap_all!(x = state.x, y = state.y ; else return -1);
        x + y
    }
    ```

-   **`let_with_err!`**: Unwraps a `Result` into a binding, or executes a typed bail block with the error accessible.
    ```rust
    let_with_err!(conn = db.connect(), Err(e) => {
        eprintln!("DB error: {e}");
        return Err(e.into());
    });
    ```

-   **`run!`**: Guard-style conditional — evaluates a condition and executes an else branch if false.
    ```rust
    run!(user.is_authenticated() else return Err(AuthError::Unauthorized));
    ```

-   **`require!`**: Multi condition guard that bails if any condition fails.
    ```rust
    require!(input.len() > 0, input.len() < 1024 ; else return Err("invalid length"));
    ```

-   **`destructure!`**: Extracts multiple fields from an `Option<Struct>` into local bindings at once.
    ```rust
    destructure!(maybe_config => { host, port, timeout } else return);
    ```

-   **`match_any!`**: Tests whether a value matches any one of multiple patterns.
    ```rust
    let is_operator = match_any!(token => '+', '-', '*', '/');
    ```

-   **`any_empty!`**: Returns `true` if any of the given collections or strings is empty.
    ```rust
    run!(!any_empty!(name, email, password) else return Err("fields required"));
    ```

-   **`either!`** / **`either_val!`**: Returns the first `Some` from a list of `Option`s.
    ```rust
    let name = either!(user.display_name(), user.username(), Some("Anonymous".to_string()));
    ```

-   **`looping!`** / **`while_loop!`**: Named loop block that produces a value via `stop!(value)`.
    ```rust
    let index = looping! {
        for (i, item) in list.iter().enumerate() {
            if item.id == target_id { stop!(i); }
        }
        stop!(usize::MAX);
    };
    ```

-   **`chain_call!`**: Threads a value through a sequence of `&mut T` modifier functions.
    ```rust
    let config = chain_call!(Config::default() => apply_env => apply_args => apply_defaults);
    ```

-   **`struct_new!`**: Derives a positional `new()` constructor matching field declaration order.
    ```rust
    struct_new! {
        pub struct Vec2 { pub x: f32, pub y: f32 }
    }
    let v = Vec2::new(1.0, 2.0);
    ```

-   **`bit_flags!`**: Defines a unit struct with associated typed bitmask constants.
    ```rust
    bit_flags! {
        pub struct Permission: u32 { READ = 0b001, WRITE = 0b010, EXEC = 0b100 }
    }
    let rw = Permission::READ | Permission::WRITE;
    ```

-   **`downcast_ref!`**: Pattern-matches a `&dyn Any` by concrete type with a fallthrough default.
    ```rust
    downcast_ref!(value.as_ref(), {
        String => s { println!("string: {s}"); },
        i32    => n { println!("number: {n}"); },
        _      => { println!("unknown type"); }
    });
    ```

-   **`enum_str!`**: Derives `as_str()` returning the variant name as `&'static str`.
    ```rust
    enum_str! {
        pub enum Direction { North, South, East, West }
    }
    assert_eq!(Direction::North.as_str(), "North");
    ```

-   **`fn_log!`**: Wraps a sync or async function with entry/exit timing instrumentation.
    ```rust
    fn_log! {
        pub async fn fetch_data(url: &str) -> Result<Vec<u8>, reqwest::Error> {
            reqwest::get(url).await?.bytes().await.map(|b| b.to_vec())
        }
    }
    ```

-   **`size_of!`**: Prints size and alignment of one or more types at runtime for layout inspection.
    ```rust
    size_of!(u8, u32, u64, MyStruct);
    ```

-   **`compile_note!`**: Emits a compiler deprecation warning at compile time without blocking compilation.
    ```rust
    compile_note!("TODO: replace this with the new config system before v2.0");
    ```

-   **`type_assert!`**: Compile time assertion that an expression matches a specific type.
    ```rust
    type_assert!(my_value => u32);
    ```