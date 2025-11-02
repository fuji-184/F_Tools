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
}
