
use ftool::*;

use std::thread;
use std::sync::Arc;
use std::time::Duration;

fn main() {

    let barrier = Arc::new(FutexThreadPause::new(4));
    let mut handles = Vec::new();

    for i in 0..4 {
        let b = Arc::clone(&barrier);
        handles.push(thread::spawn(move || {
            println!("Thread {} sedang melakukan inisialisasi...", i);
            
            thread::sleep(Duration::from_millis(i * 100)); 

            println!("Thread {} mencapai titik tunggu (Barrier)", i);
            
            b.wait();

            println!("Thread {} lepas dari barrier dan mulai memproses data", i);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Semua tugas selesai");
}