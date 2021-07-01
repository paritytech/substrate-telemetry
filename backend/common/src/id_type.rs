/// Define a type that can be used as an ID, be converted from/to the inner type,
/// and serialized/deserialized transparently into the inner type.
#[macro_export]
macro_rules! id_type {
    ($( #[$attrs:meta] )* $vis:vis struct $ty:ident ( $inner:ident ) $(;)? ) => {
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
    //! Mostly we're just checking that everything compiles OK
    //! when the macro is used as expected..

    // A basic definition is possible:
    id_type! {
        struct Foo(usize)
    }

    // We can add a ';' on the end:
    id_type! {
        struct Bar(usize);
    }

    // Visibility qualifiers are allowed:
    id_type! {
        pub struct Wibble(u64)
    }

    // Doc strings are possible
    id_type! {
        /// We can have doc strings, too
        pub(crate) struct Wobble(u16)
    }

    // In fact, any attributes can be added (common
    // derives are added already):
    id_type! {
        /// We can have doc strings, too
        #[derive(serde::Serialize)]
        #[serde(transparent)]
        pub(crate) struct Lark(u16)
    }

    #[test]
    fn create_and_use_new_id_type() {
        let _ = Foo::new(123);
        let id = Foo::from(123);
        let id_num: usize = id.into();

        assert_eq!(id_num, 123);
    }
}
