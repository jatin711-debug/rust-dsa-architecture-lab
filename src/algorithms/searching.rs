//! Binary search over a sorted slice.
//!
//! These functions borrow the input and work with non-`Copy` values. The
//! sorted-input precondition is the caller's responsibility.

/// Index of the first element that is not less than `target`.
/// Returns `slice.len()` if every element is smaller. O(log n) comparisons.
#[must_use]
pub fn lower_bound<T: Ord>(slice: &[T], target: &T) -> usize {
    let (mut lo, mut hi) = (0, slice.len());
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if slice[mid] < *target {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

/// Finds the first occurrence of `target` in a sorted slice.
/// O(log n) comparisons and O(1) space.
#[must_use]
pub fn binary_search_first<T: Ord>(slice: &[T], target: &T) -> Option<usize> {
    let index = lower_bound(slice, target);
    (slice.get(index) == Some(target)).then_some(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries_and_duplicates() {
        let values = [1, 2, 2, 2, 5];
        assert_eq!(lower_bound(&values, &0), 0);
        assert_eq!(lower_bound(&values, &2), 1);
        assert_eq!(lower_bound(&values, &3), 4);
        assert_eq!(lower_bound(&values, &9), 5);
        assert_eq!(binary_search_first(&values, &2), Some(1));
        assert_eq!(binary_search_first(&values, &3), None);
        assert_eq!(binary_search_first::<i32>(&[], &3), None);
    }
}
