use std::marker::PhantomData;
use std::ops::Deref;

pub trait RawPointer<T: ?Sized> {
    fn immutable_pointer(&self) -> *const T;
    fn mutable_pointer(&self) -> *mut T;
    fn set_immutable_pointer(&mut self, ptr: *const T);
    fn set_mutable_pointer(&mut self, ptr: *mut T);
    fn from_immutable_pointer(ptr: *const T) -> Self;
    fn from_mutable_pointer(ptr: *mut T) -> Self;
}

pub struct AliasingGuardMut<'a, T: ?Sized> {
    ptr: *mut T,
    start_addr: Option<usize>,
    end_addr: Option<usize>,
    _marker: PhantomData<&'a mut T>,
}

pub struct AliasingGuardConst<'a, T: ?Sized> {
    ptr: *const T,
    start_addr: Option<usize>,
    end_addr: Option<usize>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: ?Sized> RawPointer<T> for AliasingGuardMut<'a, T> {
    #[inline(always)]
    fn immutable_pointer(&self) -> *const T { self.ptr as *const T }
    #[inline(always)]
    fn mutable_pointer(&self) -> *mut T { self.ptr }
    fn set_immutable_pointer(&mut self, ptr: *const T) {
        self.ptr = ptr as *mut T;
    }
    fn set_mutable_pointer(&mut self, ptr: *mut T) {
        self.ptr = ptr;
    }
    fn from_immutable_pointer(ptr: *const T) -> Self { Self::from_pointer(ptr as *mut T) }
    fn from_mutable_pointer(ptr: *mut T) -> Self { Self::from_pointer(ptr) }
}

impl<'a, T: ?Sized> RawPointer<T> for AliasingGuardConst<'a, T> {
    fn immutable_pointer(&self) -> *const T { self.ptr }
    fn mutable_pointer(&self) -> *mut T { self.ptr as *mut T }
    fn set_immutable_pointer(&mut self, ptr: *const T) {
        self.ptr = ptr;
    }
    fn set_mutable_pointer(&mut self, ptr: *mut T) {
        self.ptr = ptr as *const T;
    }
    fn from_immutable_pointer(ptr: *const T) -> Self { Self::from_pointer(ptr) }
    fn from_mutable_pointer(ptr: *mut T) -> Self { Self::from_pointer(ptr as *const T) }
}

fn check_alignment<T: ?Sized>(ptr: *const T) {
    let addr = ptr as *const () as usize;
    let align = unsafe { align_of_val(&*ptr) }; 
    debug_assert!(addr % align == 0, "Pointer address {} is not aligned to {}", addr, align);
}

impl<'a, T: ?Sized> AliasingGuardConst<'a, T> {
    pub fn from_reference(value: &'a T) -> Self {
        Self {
            ptr: value as *const T,
            start_addr: None,
            end_addr: None,
            _marker: PhantomData,
        }
    }
    
    pub fn from_pointer(value: *const T) -> Self {
        debug_assert!(!value.is_null());
        check_alignment(value);
        
        Self {
            ptr: value,
            start_addr: None,
            end_addr: None,
            _marker: PhantomData,
        }
    }
    
    fn cast_guard<U>(self) -> AliasingGuardConst<'a, U> {
        let new_ptr = self.immutable_pointer().cast::<U>();
        
        AliasingGuardConst {
            ptr: new_ptr,
            start_addr: self.start_addr,
            end_addr: self.end_addr,
            _marker: PhantomData,
        }
    }
    
    fn cast_offset<U>(self, count: isize) -> AliasingGuardConst<'a, U>
    where 
        T: Sized,
        U: Sized
    {
        const {
            if std::mem::align_of::<U>() > std::mem::align_of::<T>() {
                panic!("Alignment mismatch: Target type has stricter alignment");
            }
        }
        
        let new_ptr = unsafe { 
            self.immutable_pointer()
                .cast::<U>()
                .offset(count) 
        };
        
        AliasingGuardConst::from_pointer(new_ptr)
    }
}

impl<'a, T: ?Sized> AliasingGuardMut<'a, T> {
    #[inline(always)]
    pub fn from_reference(value: &'a mut T) -> Self {
        Self {
            ptr: value as *mut T,
            start_addr: None,
            end_addr: None,
            _marker: PhantomData,
        }
    }
    
    pub fn from_pointer(value: *mut T) -> Self {
        debug_assert!(!value.is_null());
        check_alignment(value);
        
        Self {
            ptr: value,
            start_addr: None,
            end_addr: None,
            _marker: PhantomData,
        }
    }
    
    fn cast_guard<U>(self) -> AliasingGuardMut<'a, U> {
        let new_ptr = self.mutable_pointer().cast::<U>();
        
        AliasingGuardMut {
            ptr: new_ptr,
            start_addr: self.start_addr,
            end_addr: self.end_addr,
            _marker: PhantomData,
        }
    }
    
    fn cast_offset<U>(self, count: isize) -> AliasingGuardMut<'a, U>
    where 
        T: Sized,
        U: Sized
    {
        const {
            if std::mem::align_of::<U>() > std::mem::align_of::<T>() {
                panic!("Alignment mismatch: Target type has stricter alignment");
            }
        }
        
        let new_ptr = unsafe { 
            self.mutable_pointer()
                .cast::<U>()
                .offset(count) 
        };
        
        AliasingGuardMut::from_pointer(new_ptr)
    }
}

impl<'a, T: Sized> AliasingGuardMut<'a, T> {
    pub fn from_mutable_slice(slice: &'a mut [T]) -> Self {
        let ptr = slice.as_mut_ptr();
        let len = slice.len();
        let start_addr = ptr as usize;
        let end_addr = start_addr + (len * size_of::<T>());

        Self {
            ptr,
            start_addr: Some(start_addr),
            end_addr: Some(end_addr),
            _marker: PhantomData,
        }
    }
    
    pub fn bound_checked_offset(self, count: isize) -> Self {
        let new_ptr = unsafe { self.ptr.offset(count) };
        let new_addr = new_ptr as usize;

        if let (Some(start_addr), Some(end_addr)) = (self.start_addr, self.end_addr) {
            assert!(
                new_addr >= start_addr && new_addr < end_addr,
                "Out of Bounds: Offset {} is ouside of the location (Addr: {} - {})",
                count, start_addr, end_addr
            );
        }

        Self {
            ptr: new_ptr,
            start_addr: self.start_addr,
            end_addr: self.end_addr,
            _marker: PhantomData,
        }
    }
    
    pub fn bound_checked_advance(&mut self, count: isize) {
        let new_ptr = self.ptr.wrapping_offset(count);
        let new_addr = new_ptr as usize;

        if let (Some(start_addr), Some(end_addr)) = (self.start_addr, self.end_addr) {
            if new_addr < start_addr || new_addr >= end_addr {
                panic!(
                    "Out of Bounds: Advance by {} elements is outside the allocated range!\n\
                    Valid range: {} - {}\n\
                    Target address: {}",
                    count, start_addr, end_addr, new_addr
                );
            }
        }

        self.ptr = new_ptr;
    }
}

pub trait AliasingGuardExt<'a, T: ?Sized>: RawPointer<T> + Sized {

    #[inline(always)]
    fn mutable_reference(&mut self) -> &mut T {
        unsafe { &mut *self.mutable_pointer() }
    }

    #[inline(always)]
    fn immutable_reference(&self) -> &T {
        unsafe { &*self.immutable_pointer() }
    }
    
    fn close(self) {
        
    }
    
    fn cast_mutable_pointer<U>(&self) -> *mut U {
        self.mutable_pointer().cast::<U>()
    }
    
    fn cast_mutable_pointer_and_close<U>(self) -> *mut U {
        self.mutable_pointer().cast::<U>()
    }
    
    fn reference_different_type<U>(&self) -> &U 
    where 
        T: Sized, 
        U: Sized 
    {
        const {
            if size_of::<T>() != size_of::<U>() {
                panic!("Size mismatch: Source and target types must have the same size in bytes.");
            }
            if align_of::<T>() < align_of::<U>() {
                panic!("Alignment mismatch: Target type requires stricter alignment than source type.");
            }
        }

        unsafe { &*self.immutable_pointer().cast::<U>() }
    }
    
    fn mutable_reference_different_type<U>(&mut self) -> &mut U 
    where 
        T: Sized, 
        U: Sized 
    {
        const {
            if size_of::<T>() != size_of::<U>() {
                panic!("Size mismatch: Source and target types must have the same size in bytes.");
            }
            if align_of::<T>() < align_of::<U>() {
                panic!("Alignment mismatch: Target type requires stricter alignment than source type.");
            }
        }

        unsafe { &mut *self.mutable_pointer().cast::<U>() }
    }
    
    fn cast_immutable_reference_array<U, const N: usize>(&self) -> &[U; N]
    where
        T: Sized,
        U: Sized,
    {
        const {
            let total_target_size = size_of::<U>() * N;
            if size_of::<T>() != total_target_size {
                panic!("Size mismatch: The source type size does not match the total size of the requested array.");
            }
            if align_of::<T>() < align_of::<U>() {
                panic!("Alignment mismatch: Target element type requires stricter alignment than source type.");
            }
        }

        unsafe { & *self.immutable_pointer().cast::<[U; N]>() }
    }
    
    fn cast_immutable_reference_slice<U>(&self, len: usize) -> &[U]
    where
        T: Sized,
        U: Sized,
    {
        const {
            if align_of::<T>() < align_of::<U>() {
                panic!("Alignment mismatch: Target element type requires stricter alignment.");
            }
        }

        assert!(
            len * size_of::<U>() <= size_of::<T>(),
            "Runtime Error: Requested slice length exceeds source memory size."
        );

        unsafe { std::slice::from_raw_parts(self.immutable_pointer().cast::<U>(), len) }
    }
    
    fn cast_mutable_reference_array<U, const N: usize>(&mut self) -> &mut [U; N]
    where
        T: Sized,
        U: Sized,
    {
        const {
            let total_target_size = size_of::<U>() * N;
            if size_of::<T>() != total_target_size {
                panic!("Size mismatch: The source type size does not match the total size of the requested array.");
            }
            if align_of::<T>() < align_of::<U>() {
                panic!("Alignment mismatch: Target element type requires stricter alignment than source type.");
            }
        }

        unsafe { &mut *self.mutable_pointer().cast::<[U; N]>() }
    }
    
    fn cast_mutable_reference_slice<U>(&mut self, len: usize) -> &mut [U]
    where
        T: Sized,
        U: Sized,
    {
        const {
            if align_of::<T>() < align_of::<U>() {
                panic!("Alignment mismatch: Target element type requires stricter alignment.");
            }
        }

        assert!(
            len * size_of::<U>() <= size_of::<T>(),
            "Runtime Error: Requested slice length exceeds source memory size."
        );

        unsafe { std::slice::from_raw_parts_mut(self.mutable_pointer().cast::<U>(), len) }
    }
    
    unsafe fn offset(self, count: isize) -> Self 
    where 
        T: Sized,
        Self: RawPointer<T>
    {
        let new_ptr = unsafe { self.immutable_pointer().offset(count) };
        Self::from_immutable_pointer(new_ptr as *mut T)
    }
    
    unsafe fn advance(&mut self, count: isize) 
    where T: Sized 
    {
        self.set_mutable_pointer(unsafe { self.mutable_pointer().offset(count) });
    }
    
}

impl<'a, T: ?Sized> AliasingGuardExt<'_, T> for AliasingGuardMut<'a, T> {}
impl<'a, T: ?Sized> AliasingGuardExt<'_, T> for AliasingGuardConst<'a, T> {}


impl<'a, T: ?Sized> Deref for AliasingGuardConst<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        assert!(!self.ptr.is_null(), "Attempted to dereference a null AliasingGuardConst");
        
        unsafe { &*self.ptr }
    }
}

impl<'a, T: ?Sized> Deref for AliasingGuardMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        assert!(!self.ptr.is_null(), "Attempted to dereference a null AliasingGuardMut");
        
        unsafe { &*self.ptr }
    }
}

struct a {
    b: String
}
impl a {
     #[inline(always)]
    fn a(&mut self)  {
        let mut ptr = &raw mut self.b;
        let reff = unsafe { &mut *ptr };
        *reff = String::from("hello 2");
    }
    
    #[inline(always)]
    fn b(&mut self) {
        let mut guard = AliasingGuardMut::from_reference(&mut self.b);
        let b = guard.mutable_reference();
        *b = String::from("hello 2");
    }
}

#[unsafe(no_mangle)]
fn a1(mut a: a) {
    a.a();
}

#[unsafe(no_mangle)]
fn a2(mut a: a) {
    a.b();
}


fn main() {
    let mut a = String::from("hello");
    let mut ptr = std::ptr::NonNull::new(&raw mut a).unwrap();
    
    unsafe {
        let tes = ptr.as_mut();
        let tes2 = ptr.as_mut();
        
        // this will compile but it causes UB, checked in Miri
        *tes = String::from("hello 2");
        println!("{}", *tes2);
    }

    let mut guard = AliasingGuardMut::from_reference(&mut a);

    
    let b = guard.mutable_reference();
    *b = String::from("reference from raw ptr");

    // these will give compile time error
    
    //let c_illegal = guard.immutable_reference();
    //let d_illegal = guard.mutable_reference();

    *b = String::from("reference from raw ptr 2");
    
    let e = guard.immutable_reference();
    
    // this will give compile time error
    
    // let f_illegal = guard.mutable_reference();

    println!("{}", *e);

    // drop(guard) or guard.close() to close the guard without waiting an end of scope
    
    println!("{}", a);
    
    //guard.close();
    
    let mut guard = AliasingGuardMut::from_reference(&mut a);

    // these will cause compile time error
    
    //let g_illegal = guard.reference_different_type::<&i64>();
    //let h_illegal = guard.mutable_reference_different_type::<&mut i64>();
 
    //let i_illegal = guard.cast_immutable_reference_array::<i64, 1024>();
    //let j_illegal = guard.cast_mutable_reference_array::<i64, 1024>();
    
    // these will cause runtime panic
    
    //let k_illegal = guard.cast_immutable_reference_slice::<i64>(1024);
    //let l_illegal = guard.cast_mutable_reference_slice::<i64>(1024);
    
    let mut numbers = [10u32, 20, 30, 40];
    
    let mut guard = AliasingGuardMut::from_mutable_slice(&mut numbers);
    
    // move pointer to point index 1
    guard.bound_checked_advance(1); 
    println!("Index 1: {}", *guard); // print 20
    
    // this will cause runtime panic
    
    //guard.bound_checked_advance(100); 
    
    // this will cause compile time error
    
    //let illegal_m = guard.cast_offset::<u64>(1);

    
}



