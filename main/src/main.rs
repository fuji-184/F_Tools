#![feature(allocator_api)]

use ftool::*;
use std::alloc::{AllocError, Allocator, Layout};
use std::ptr::NonNull;

#[repr(C)]
pub struct VmmHeader {
    pub magic:       u64,
    pub version:     u32,
    pub total_size:  u64,
    pub checksum:    u64,
    pub bump_offset: u64,
    pub slot_vec:    u64,
    pub slot_string: u64,
}

pub struct PersistentVmm {
    pub memmap: MemMap,
}

impl PersistentVmm {
    pub fn new(memmap: MemMap) -> Self {
        unsafe {
            let header = &mut *(memmap.ptr as *mut VmmHeader);
            let header_size = std::mem::size_of::<VmmHeader>() as u64;
            if header.magic != 0x46544F4F4C53 {
                header.magic       = 0x46544F4F4C53;
                header.version     = 1;
                header.total_size  = memmap.len as u64;
                header.bump_offset = header_size;
                header.slot_vec    = 0;
                header.slot_string = 0;
            }
        }
        Self { memmap }
    }

    pub fn header(&self) -> &mut VmmHeader {
        unsafe { &mut *(self.memmap.ptr as *mut VmmHeader) }
    }

    pub fn offset_of(&self, ptr: *const u8) -> u64 {
        ptr as usize as u64 - self.memmap.ptr as usize as u64
    }

    pub fn ptr_at<T>(&self, offset: u64) -> *mut T {
        (self.memmap.ptr as usize + offset as usize) as *mut T
    }
}

unsafe impl Allocator for PersistentVmm {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let header  = self.header();
        let current = header.bump_offset as usize;
        let align   = layout.align();
        let aligned = current.checked_add(align - 1).ok_or(AllocError)? & !(align - 1);
        let end     = aligned.checked_add(layout.size()).ok_or(AllocError)?;
        if end > self.memmap.len { return Err(AllocError); }
        header.bump_offset = end as u64;
        let ptr = unsafe { NonNull::new_unchecked(self.memmap.ptr.add(aligned)) };
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
    }

    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {}
}

#[repr(C, align(8))]
struct CollectionMeta {
    len:         u64,
    cap:         u64,
    data_offset: u64,
}

fn collection_init<T>(vmm: &PersistentVmm, cap: usize) -> u64 {
    let data_block  = vmm.allocate(Layout::array::<T>(cap).unwrap()).unwrap();
    let data_offset = vmm.offset_of(data_block.as_ptr() as *const u8);

    let meta_block = vmm.allocate(Layout::new::<CollectionMeta>()).unwrap();
    let meta_ptr   = meta_block.as_ptr() as *mut CollectionMeta;
    let offset     = vmm.offset_of(meta_ptr as *const u8);

    unsafe {
        meta_ptr.write(CollectionMeta { len: 0, cap: cap as u64, data_offset });
    }

    offset
}

fn collection_len(vmm: &PersistentVmm, offset: u64) -> usize {
    unsafe { (*vmm.ptr_at::<CollectionMeta>(offset)).len as usize }
}

fn collection_set_len(vmm: &PersistentVmm, offset: u64, len: usize) {
    unsafe { (*vmm.ptr_at::<CollectionMeta>(offset)).len = len as u64; }
}

fn collection_cap(vmm: &PersistentVmm, offset: u64) -> usize {
    unsafe { (*vmm.ptr_at::<CollectionMeta>(offset)).cap as usize }
}

fn collection_set_cap(vmm: &PersistentVmm, offset: u64, cap: usize) {
    unsafe { (*vmm.ptr_at::<CollectionMeta>(offset)).cap = cap as u64; }
}

fn collection_data_ptr<T>(vmm: &PersistentVmm, offset: u64) -> *mut T {
    let data_offset = unsafe { (*vmm.ptr_at::<CollectionMeta>(offset)).data_offset };
    vmm.ptr_at::<T>(data_offset)
}

fn vec_from_mmap<'vmm, T>(vmm: &'vmm PersistentVmm, offset: u64) -> Vec<T, &'vmm PersistentVmm> {
    let len = collection_len(vmm, offset);
    let cap = collection_cap(vmm, offset);
    let ptr = collection_data_ptr::<T>(vmm, offset);
    unsafe { Vec::from_raw_parts_in(ptr, len, cap, vmm) }
}

pub struct PersistentVec<'vmm, T> {
    vec:    Vec<T, &'vmm PersistentVmm>,
    vmm:    &'vmm PersistentVmm,
    offset: u64,
}

impl<'vmm, T> PersistentVec<'vmm, T> {
    pub fn open(vmm: &'vmm PersistentVmm, slot: &mut u64, cap: usize) -> Self {
        let offset = if *slot == 0 {
            let off = collection_init::<T>(vmm, cap);
            *slot = off;
            off
        } else {
            *slot
        };
        Self { vec: vec_from_mmap(vmm, offset), vmm, offset }
    }
}

impl<'vmm, T> std::ops::Deref for PersistentVec<'vmm, T> {
    type Target = Vec<T, &'vmm PersistentVmm>;
    fn deref(&self) -> &Self::Target { &self.vec }
}

impl<'vmm, T> std::ops::DerefMut for PersistentVec<'vmm, T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.vec }
}

impl<'vmm, T> Drop for PersistentVec<'vmm, T> {
    fn drop(&mut self) {
        collection_set_len(self.vmm, self.offset, self.vec.len());
        collection_set_cap(self.vmm, self.offset, self.vec.capacity());
        let vec = std::mem::replace(&mut self.vec, Vec::new_in(self.vmm));
        std::mem::forget(vec);
        self.vmm.memmap.flush().unwrap();
    }
}

pub struct PersistentString<'vmm> {
    inner: PersistentVec<'vmm, u8>,
}

impl<'vmm> PersistentString<'vmm> {
    pub fn open(vmm: &'vmm PersistentVmm, slot: &mut u64, cap: usize) -> Self {
        Self { inner: PersistentVec::open(vmm, slot, cap) }
    }

    pub fn push_str(&mut self, s: &str) {
        self.inner.extend_from_slice(s.as_bytes());
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.inner).unwrap()
    }
}

impl<'vmm> std::ops::Deref for PersistentString<'vmm> {
    type Target = str;
    fn deref(&self) -> &str { self.as_str() }
}

impl<'vmm> std::ops::DerefMut for PersistentString<'vmm> {
    fn deref_mut(&mut self) -> &mut str {
        std::str::from_utf8_mut(&mut self.inner).unwrap()
    }
}

impl<'vmm> std::fmt::Display for PersistentString<'vmm> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'vmm> std::fmt::Debug for PersistentString<'vmm> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

fn main() {
    let memmap = MemMap::open("./a.bin", true, Some(1024 * 8)).unwrap();
    let vmm    = PersistentVmm::new(memmap);

    {
        let header = vmm.header();
        let mut v  = PersistentVec::<u32>::open(&vmm, &mut header.slot_vec, 64);
        v.push(42);
        println!("vec: {:?}", &v[..]);
    }

    {
        let header = vmm.header();
        let mut s  = PersistentString::open(&vmm, &mut header.slot_string, 256);
        s.push_str("hello");
        println!("string: {}", &*s);
    }
}