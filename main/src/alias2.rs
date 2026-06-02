use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct AliasingGuardMut<'a, T: ?Sized> {
    ptr: NonNull<T>,

    // SAFETY:
    // This models exclusive mutable ownership over `T` for lifetime `'a`.
    //
    // The guard conceptually behaves like it owns an `&'a mut T`,
    // which prevents aliasing mutable borrows through Rust's borrow checker.
    //
    // `PhantomData<&'a mut T>` is important because:
    // - it enforces invariance over `T`
    // - it tells the compiler this type semantically contains `&mut T`
    // - it enables borrow checking rules for aliasing/exclusivity
    // - it prevents multiple mutable guards existing simultaneously in safe code
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: ?Sized> AliasingGuardMut<'a, T> {
    #[inline(always)]
    pub fn from_reference(value: &'a mut T) -> Self {
        Self {
            // SAFETY:
            // `NonNull::from` is safe because `&mut T` is guaranteed:
            // - non-null
            // - properly aligned
            // - valid for reads/writes for `'a`
            ptr: NonNull::from(value),

            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn immutable_reference(&self) -> &T {
        // SAFETY:
        // The original `&mut T` guarantees:
        // - pointer validity
        // - proper alignment
        // - initialized memory
        //
        // Returning `&T` from `&self` is safe because:
        // - immutable references may alias other immutable references
        // - Rust reference rules prevent obtaining `&mut self` simultaneously with this reference in safe code
        unsafe { self.ptr.as_ref() }
    }

    #[inline(always)]
    pub fn mutable_reference(&mut self) -> &mut T {
        // SAFETY:
        // `&mut self` guarantees exclusive access to the guard.
        //
        // Because the guard semantically owns an exclusive `&mut T`,
        // this ensures no competing mutable references can exist
        // through this API in safe Rust.
        //
        // WARNING:
        // Raw pointers previously extracted from this guard may still
        // exist and can violate aliasing rules if used incorrectly.
        // Safe Rust callers cannot trigger UB here, but unsafe callers can.
        unsafe { self.ptr.as_mut() }
    }

    #[inline(always)]
    pub fn with_immutable_reference<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        // SAFETY:
        // Same reasoning as `immutable_reference`.
        //
        // The reference is scoped to the closure call,
        // preventing it from escaping accidentally.
        unsafe { f(self.ptr.as_ref()) }
    }

    #[inline(always)]
    pub fn with_mutable_reference<R>(&mut self, f: impl FnOnce(&mut T) -> R) -> R {
        // SAFETY:
        // Same reasoning as `mutable_reference`.
        //
        // The mutable reference is scoped to the closure execution,
        // which helps reduce accidental misuse duration.
        unsafe { f(self.ptr.as_mut()) }
    }

    #[inline(always)]
    pub fn with_immutable_pointer<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
        // SAFETY:
        // - Rust reference rules prevent obtaining `&mut self` simultaneously with this reference in safe code
        //
        // In particular:
        // - The immutable raw pointer is scoped to the closure execution
        // which makes able to create `&mut` without invalidating the pointers
        // - It prevents calling immutable raw pointer while `&mut` is still active because it violates the aliasing rules
        f(self.ptr.as_ptr())
    }

    #[inline(always)]
    pub fn with_mutable_pointer<R>(&mut self, f: impl FnOnce(*mut T) -> R) -> R {
        // SAFETY:
        // - Rust reference rules prevent obtaining `&mut self` simultaneously with this reference in safe code
        //
        // In particular:
        // - The mutable raw pointer is scoped to the closure execution
        // which makes able to create `&` or `&mut` without invalidating the pointers
        // - It prevents calling mutable raw pointer while `&` or `&mut` is still active because it violates the aliasing rules
        f(self.ptr.as_ptr())
    }

    #[inline(always)]
    pub unsafe fn as_ptr(&mut self) -> *mut T {
        // SAFETY:
        // This exists to make if closure based pointer is not enough, then this unsafe method can be used
        // Returning raw pointers is safe by itself.
        //
        // However, once the pointer escapes, this type can no longer
        // enforce aliasing guarantees.
        //
        // The caller must ensure:
        // - no invalid reference/raw-pointer combinations are used
        // - no aliasing UB occurs
        // - do not write to the pointer while `&` or `&mut` to same memory is still active
        // - do not read the pointer while `&mut` to same memory is still active
        // - be aware that `&mut` creation that points to same address of this pointer will invalidate this pointer
        // - pointer is not used after underlying value becomes invalid
        self.ptr.as_ptr()
    }

    #[inline(always)]
    pub fn close(self) {
        // SAFETY:
        // Consuming `self` ends the guard lifetime early.
        //
        // This can be useful to release the conceptual mutable borrow
        // before the surrounding scope ends.
    }
}

fn main() {
    let mut a = String::from("hello");
   
    let mut guard = AliasingGuardMut::from_reference(&mut a);

    //let ptr = unsafe { guard.as_ptr() };
    
    let b = guard.mutable_reference();
    
    guard.with_mutable_pointer(|ptr| unsafe {
        *ptr = String::from("hello");
    });
    
    //unsafe { *ptr = String::from("hello 2") };
    *b = String::from("hello 3");
    
    println!("{}", *b);

    // this will give compile time error
    //let mut guard2 = AliasingGuardMut::from_reference(&mut a);


    //// let b = guard.mutable_reference();
   //// *b = String::from("reference from raw ptr");
    
    // these will give compile time error
    
    //let c_illegal = guard.immutable_reference();
    //let d_illegal = guard.mutable_reference();

  ////  *b = String::from("reference from raw ptr 2");
    
 ////   let e = guard.immutable_reference();
    
    // this will give compile time error
    
    // let f_illegal = guard.mutable_reference();
    
  ////  println!("{}", *e);
    
  ////  let g = guard.mutable_reference();

    /* this will give compile time error
    guard.with_immutable_pointer(|ptr| {
        println!("{}", unsafe { &*ptr });
    });
    */
    
    /* this will give compile time error
    guard.with_mutable_pointer(|ptr| unsafe {
        *ptr = String::from("hello");
    });
    */
    
    /* this will give compile time error
    guard.with_immutable_reference(|reff| {
        println!("{}", *reff);
    });
    */
    
    /* this will give compile time error
    guard.with_mutable_reference(|reff| {
        *reff = String::from("hello");
    });
    */
    
  ////  *g = String::from("reference from raw ptr 3");

    // drop(guard) or guard.close() to close the guard without waiting an end of scope
    
  ////  println!("{}", a);
    
    
}