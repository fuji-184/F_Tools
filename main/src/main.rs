use ftool::defer;

fn main() {
    for i in 0..3 {
        defer! {
            println!("defer {}", i);
        }
        println!("Hello, world!");
    }

    ftool::dlog!("debug");

    let start = std::time::Instant::now();
    println!("bench");
    let time = start.elapsed();
    println!("time: {}", time.as_nanos());

    ftool::bench!(10, {
        println!("bench");
    });
}
