
use ftool::*;

#[tokio::main]
async fn main() {
    let mut counter = 0;
    let data = vec![1, 2, 3];

    local_async_scope(|s| {
        s.spawn(async {
            println!("Thread {:?}, data: {:?}", std::thread::current().id(), data);
        });

        s.spawn(async {
            println!("Thread {:?}, data len: {}", std::thread::current().id(), data.len());
        });
        counter += 10;
    }).await;

}