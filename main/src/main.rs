use ftool::*;

fn main() {
    let wg = FutexWaitGroup::new();
    
    wg.add(4);

    for i in 0..4 {
        let wg_clone = wg.clone();
        std::thread::spawn(move || {
            println!("Worker {} mulai bekerja...", i);
            
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            println!("Worker {} selesai", i);
            wg_clone.done(); 
        });
    }

    println!("Menunggu semua worker...");
    wg.wait();
    println!("Semua tugas beres. Lanjut");
}