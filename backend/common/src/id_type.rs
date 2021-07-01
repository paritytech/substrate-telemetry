/// Define a type that can be used as an ID, be converted from/to the inner type,
/// and serialized/deserialized transparently into the inner type.
#[macro_export]
macro_rules! id_type {
    ($( #[$attrs:meta] )* $vis:vis $ty:ident ( $inner:ident ) $(;)? ) => {
        #[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
        $( #[$attrs] )*
        $vis struct $ty($inner);

        impl $ty {
            #[allow(dead_code)]
            pub fn new(inner: $inner) -> Self {
                Self(inner)
            }
        }

        impl From<$inner> for $ty {
            fn from(inner: $inner) -> Self {
                Self(inner)
            }
        }

        impl From<$ty> for $inner {
            fn from(ty: $ty) -> Self {
                ty.0
            }
        }
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn create_and_use_new_id_type() {
        id_type! {
            Foo(usize)
        };
        let _ = Foo::new(123);
        let id = Foo::from(123);
        let _: usize = id.into();

        // Check that these don't lead to compile errors:
        id_type! {
            Bar(usize);
        };
        id_type! {
            pub Wibble(u64)
        };
        id_type! {
            /// We can have doc strings, too
            pub(crate) Wobble(u16)
        };
    }
}
