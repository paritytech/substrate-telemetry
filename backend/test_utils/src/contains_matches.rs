/**
This macro checks to see whether an iterable container contains each of the
match items given, in the order that they are given in (but not necessarily
contiguous, ie other items may be interspersed between the ones we're looking
to match).

Similar to `matches!`.

```
enum Item {
    Foo { a: usize },
    Bar(bool),
    Wibble
}

use Item::*;

let does_contain: bool = test_utils::contains_matches!(
    vec![Foo { a: 2 }, Wibble, Bar(true), Foo { a: 100 }],
    Foo { a: 2 } | Foo { a: 3 },
    Bar(true),
    Foo {..}
);

assert!(does_contain);
```
*/
#[macro_export]
macro_rules! contains_matches {
    ($expression:expr, $( $( $pattern:pat )|+ $( if $guard:expr )? ),+ $(,)?) => {{
        let mut items = $expression.into_iter();

        // For each pattern we want to match, we consume items until
        // we find the first match, and then break the loop and do the
        // same again with the next pattern. If we run out of items, we
        // set the validity to false and stop trying to match. Else, we
        // match againse each of the patterns and return true.
        let mut is_valid = true;
        $(
            while is_valid {
                let item = match items.next() {
                    Some(item) => item,
                    None => {
                        is_valid = false;
                        break;
                    }
                };

                match item {
                    $( $pattern )|+ $( if $guard )? => break,
                    _ => continue
                }
            }
        )+

        is_valid
    }}
}

/**
This macro checks to see whether an iterable container contains each of the
match items given, in the order that they are given in (but not necessarily
contiguous, ie other items may be interspersed between the ones we're looking
to match).

Panics if this is not the case.
```
enum Item {
    Foo { a: usize },
    Bar(bool),
    Wibble
}

use Item::*;

test_utils::assert_contains_matches!(
    vec![Foo { a: 2 }, Wibble, Bar(true), Foo { a: 100 }],
    Foo { a: 2 },
    Bar(true),
    Foo {..}
);
```
*/
#[macro_export]
macro_rules! assert_contains_matches {
    ($expression:expr, $( $( $pattern:pat )|+ $( if $guard:expr )? ),+ $(,)?) => {
        let does_contain_matches = $crate::contains_matches!(
            $expression,
            $( $( $pattern )|+ $( if $guard )? ),+
        );

        assert!(does_contain_matches);
    }
}
