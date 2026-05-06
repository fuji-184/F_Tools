
use ftool::*;

use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicU64;

struct AppState {
    counter: AtomicU64,
    last_updated: u64,
}

fn main() {
    let mmap = MemMap::open("config.txt", true, Some(1024 * 1024)).unwrap();
    
    let vmm = PersistentVmm::new(mmap);

    let state: *mut AppState = vmm.get_root_mut_ref();

    unsafe {
        (*state).counter.fetch_add(1, Ordering::SeqCst);
        println!("Counter saat ini: {}", (*state).counter.load(Ordering::Relaxed));
    }
}