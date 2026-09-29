//! Declarative macros (`macro_rules!`): metaprogramming.
//!
//! Macros are code that writes code: they match token patterns and emit
//! replacement tokens at compile time. Three techniques are demonstrated:
//!
//! - **Repetition** (`$($x:expr),*`): build a container literal like
//!   [`dynarray!`] and [`hashmap!`].
//! - **Recursion**: a macro that calls itself to consume a variable-length
//!   argument list ([`sum_of!`]).
//! - **Hygiene via `$crate`**: macros must expand to paths that work from any
//!   caller's scope, which is what `$crate::...` is for.
//!
//! # Examples
//!
//! ```
//! use rust_learning_and_dsa::{dynarray, hashmap};
//!
//! let arr = dynarray![1, 2, 3];
//! assert_eq!(arr.len(), 3);
//!
//! let map = hashmap! { "one" => 1, "two" => 2 };
//! assert_eq!(map.get(&"two"), Some(&2));
//! ```

/// Builds a [`DynamicArray`](crate::linear::DynamicArray) from a list of
/// values, mirroring `vec!`.
///
/// ```
/// use rust_learning_and_dsa::{dynarray, linear::DynamicArray};
///
/// let arr = dynarray![10, 20, 30];
/// assert_eq!(arr[1], 20);
/// let empty: DynamicArray<i32> = dynarray![];
/// assert!(empty.is_empty());
/// ```
#[macro_export]
macro_rules! dynarray {
    // Empty case: an empty dynamic array.
    () => {
        $crate::linear::DynamicArray::new()
    };
    // One or more elements, with an optional trailing comma.
    ($($elem:expr),* $(,)?) => {{
        let mut arr = $crate::linear::DynamicArray::new();
        $(arr.push($elem);)*
        arr
    }};
}

/// Builds a [`HashMap`](crate::map::HashMap) from `key => value` pairs,
/// mirroring `std::collections::HashMap`'s literal syntax.
///
/// ```
/// use rust_learning_and_dsa::hashmap;
///
/// let map = hashmap! { "a" => 1, "b" => 2 };
/// assert_eq!(map.get(&"b"), Some(&2));
/// ```
#[macro_export]
macro_rules! hashmap {
    () => {
        $crate::map::HashMap::new()
    };
    ($($key:expr => $value:expr),* $(,)?) => {{
        let mut map = $crate::map::HashMap::new();
        $(map.insert($key, $value);)*
        map
    }};
}

/// Sums any number of integer expressions at compile time via recursion.
///
/// The macro matches one expression (`$head`) and recurses on the rest
/// (`$($tail),*`), terminating at the empty base case.
///
/// ```
/// use rust_learning_and_dsa::sum_of;
///
/// assert_eq!(sum_of!(), 0);
/// assert_eq!(sum_of!(1), 1);
/// assert_eq!(sum_of!(1, 2, 3, 4), 10);
/// ```
#[macro_export]
macro_rules! sum_of {
    () => {
        0
    };
    ($head:expr $(, $tail:expr)* $(,)?) => {
        $head + $crate::sum_of!($($tail),*)
    };
}

/// Builds a sorted `Vec` from a list of `Ord` values, using `crate::algorithms::quicksort` —
/// a macro that composes the library.
///
/// ```
/// use rust_learning_and_dsa::sorted_vec;
///
/// let v = sorted_vec![3, 1, 2];
/// assert_eq!(v, vec![1, 2, 3]);
/// ```
#[macro_export]
macro_rules! sorted_vec {
    ($($elem:expr),* $(,)?) => {{
        let mut v = vec![$($elem),*];
        $crate::algorithms::quicksort(&mut v);
        v
    }};
}

#[cfg(test)]
mod tests {
    use crate::linear::DynamicArray;
    use crate::map::HashMap;

    #[test]
    fn dynarray_macro() {
        let arr = dynarray![1, 2, 3];
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], 1);

        let empty: DynamicArray<i32> = dynarray![];
        assert!(empty.is_empty());

        // Trailing comma is allowed.
        let arr = dynarray![1, 2,];
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn dynarray_macro_with_expressions() {
        let arr = dynarray![1 + 1, 2 * 3];
        assert_eq!(arr[0], 2);
        assert_eq!(arr[1], 6);
    }

    #[test]
    fn hashmap_macro() {
        let map = hashmap! { "a" => 1, "b" => 2 };
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&"a"), Some(&1));

        let empty: HashMap<&str, i32> = hashmap! {};
        assert!(empty.is_empty());
    }

    #[test]
    fn sum_of_recursion() {
        assert_eq!(sum_of!(), 0);
        assert_eq!(sum_of!(5), 5);
        assert_eq!(sum_of!(1, 2, 3, 4, 5), 15);
    }

    #[test]
    fn sorted_vec_macro() {
        let v = sorted_vec![9, 1, 8, 2];
        assert_eq!(v, vec![1, 2, 8, 9]);
    }

    #[test]
    fn macros_compose_with_library() {
        // hashmap! + dynarray! feeding the library's own types.
        let mut map = hashmap! { 1 => dynarray![10, 20], 2 => dynarray![30] };
        map.get_mut(&1).expect("key 1 present").push(99);
        assert_eq!(map.get(&1).expect("key 1 present").len(), 3);
    }
}
