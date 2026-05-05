fn iterate<T, F>(mut x: T, mut f: F) -> impl Iterator<Item = T>
where
    T: Clone,
    F: FnMut(T) -> T,
{
    std::iter::from_fn(move || {
        let old = x.clone();
        let new_x = f(x.clone());
        x = new_x;
        Some(old)
    })
}

fn main() {
    ftool::bench!(1, {
    let powers_of_2 = iterate(1, |x| x * 2).take(10);
    for v in powers_of_2 {
        print!("{v} ");
    }
    });
    
    ftool::thread_local_memo!(SQUARE_CACHE, 10, |x: i32| {
        println!("Calculating square for {}...", x);
        x * x
    }, i32, i32);

    // Cara panggil
    let res1 = SQUARE_CACHE.with(|m| m.borrow_mut().call(5));
    let res2 = SQUARE_CACHE.with(|m| m.borrow_mut().call(5)); // Langsung return

    println!("Res 1: {}, Res 2: {}", res1, res2);
}
