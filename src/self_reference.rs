
/*
Self-referential data container to safely bundle an owned object alongside references to itself.

This mechanism bypasses the Rust borrow checker's strict restrictions against objects holding 
internal references to their own heap-allocated fields. It is primarily used to clean up complex 
data architectures—such as pairing an owned string buffer with a zero-copy structured parser, 
or attaching an arena allocator block to index nodes that point inside it—by anchoring the data 
source in a stable heap location and erasing lifetime parameters to provide a single, movable type.
*/

use std::marker::PhantomData;
use std::ptr::NonNull;

pub trait Yokeable<'a> {
    type Output: 'a;
}

pub struct SelfRef<T, F: for<'a> Yokeable<'a>> {
    owner: Box<T>,
    refs: NonNull<<F as Yokeable<'static>>::Output>,
    _marker: PhantomData<F>,
}

impl<T, F: for<'a> Yokeable<'a>> SelfRef<T, F> {
    pub fn new<Constructor>(data: T, make_dep: Constructor) -> Self
    where
        Constructor: for<'a> FnOnce(&'a T) -> <F as Yokeable<'a>>::Output,
    {
        let owner = Box::new(data);

        let dep = unsafe {
            let data_ref = &*(owner.as_ref() as *const T);
            make_dep(data_ref)
        };

        let dep_static = unsafe {
            std::mem::transmute_copy::<<F as Yokeable<'_>>::Output, <F as Yokeable<'static>>::Output>(&dep)
        };
        
        std::mem::forget(dep);

        Self {
            owner,
            refs: unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(dep_static))) },
            _marker: PhantomData,
        }
    }
    
    pub fn get_owner(&self) -> &T {
        self.owner.as_ref()
    }

    pub fn get_refs<'a>(&'a self) -> &'a <F as Yokeable<'a>>::Output {
        unsafe {
            std::mem::transmute(self.refs.as_ref())
        }
    }
    
    pub fn update_field<U>(&mut self, updater: U)
    where
        U: for<'a> FnOnce(&'a mut T, &'a mut <F as Yokeable<'a>>::Output),
    {
        unsafe {
            let owner_ptr = self.owner.as_mut() as *mut T;
            let dep_ptr = self.refs.as_ptr() as *mut <F as Yokeable<'static>>::Output;

            let dep_ref = std::mem::transmute::<
                &mut <F as Yokeable<'static>>::Output,
                &mut <F as Yokeable<'_>>::Output
            >(&mut *dep_ptr);

            updater(&mut *owner_ptr, dep_ref);
        }
    }
}

impl<T, F: for<'a> Yokeable<'a>> Drop for SelfRef<T, F> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.refs.as_ptr());
        }
    }
}

impl<T: std::fmt::Debug, F: for<'a> Yokeable<'a>> std::fmt::Debug for SelfRef<T, F> 
where 
    for<'a> <F as Yokeable<'a>>::Output: std::fmt::Debug 
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SelfRef")
            .field("owner", &self.owner)
            .field("refs", &self.get_refs())
            .finish()
    }
}

#[macro_export]
macro_rules! self_ref {
    (
        $(#[$main_meta:meta])*
        $vis:vis struct $struct_name:ident {
            $( $field_vis:vis $field_name:ident : $field_type:ty ),* $(,)?
        }
        => 
        $(#[$ref_meta:meta])*
        { $( $ref_field_vis:vis $ref_field:ident : $ref_type:ty ),* $(,)? }
    ) => {
        $(#[$main_meta])*
        #[derive(Debug)]
        $vis struct $struct_name {
            $( $field_vis $field_name: $field_type, )*
        }

        paste::paste! {
            $(#[$ref_meta])*
            #[derive(Debug)]
            $vis struct [<$struct_name Refs>]<'a> {
                $( $ref_field_vis $ref_field: $ref_type, )*
            }

            impl<'a> $crate::Yokeable<'a> for $struct_name {
                type Output = [<$struct_name Refs>]<'a>;
            }
        }
    };
}

#[macro_export]
macro_rules! declare_self_ref {
    (
        $struct_name:ident {
            $( $field:ident : $val:expr ),* $(,)?
        }
        => 
        { $( $ref_field:ident : &$source:ident ),* $(,)? }
    ) => {
        paste::paste! {
            $crate::SelfRef::<$struct_name, $struct_name>::new(
                $struct_name {
                    $( $field : $val, )*
                }, 
                |d| {
                    [<$struct_name Refs>] {
                        $( $ref_field : &d.$source, )*
                    }
                }
            )
        }
    };
}

#[macro_export]
macro_rules! update_self_ref_field {
    ($sref:ident, $owner_f:ident => [ $( $ref_f:ident : $t:ty ),* $(,)? ], $val:expr) => {
        $sref.update_field(|owner, refs| {
            owner.$owner_f = $val;
            unsafe {
                $(
                    let coerced: $t = owner.$owner_f.as_ref();
                    refs.$ref_f = std::mem::transmute(coerced);
                )*
            }
        });
    };

    ($sref:ident, $owner_f:ident => $ref_f:ident : $t:ty, $val:expr) => {
        $crate::update_self_ref_field!($sref, $owner_f => [$ref_f : $t], $val);
    };
}


ftest::test!(self_ref_tests, {
 
    self_ref! {
        pub struct TestData {
            pub value: String,
        }
        =>
        {
            pub value_ref: &'a str,
        }
    }

    test_create_and_read_self_ref {
        let sref = declare_self_ref!(TestData {
            value: "hello".to_string(),
        } => {
            value_ref: &value,
        });

        assert_eq!(sref.get_owner().value, "hello");
        assert_eq!(sref.get_refs().value_ref, "hello");
    }

    test_update_field_self_ref {
        let mut sref = declare_self_ref!(TestData {
            value: "initial".to_string(),
        } => {
            value_ref: &value,
        });

        update_self_ref_field!(sref, value => [value_ref: &str], "updated".to_string());

        assert_eq!(sref.get_owner().value, "updated");
        assert_eq!(sref.get_refs().value_ref, "updated");
    }

    test_debug_formatting {
        let sref = declare_self_ref!(TestData {
            value: "debug".to_string(),
        } => {
            value_ref: &value,
        });

        let debug_str = format!("{:?}", sref);
        assert!(debug_str.contains("SelfRef"));
        assert!(debug_str.contains("owner"));
        assert!(debug_str.contains("refs"));
        assert!(debug_str.contains("debug"));
    }
});