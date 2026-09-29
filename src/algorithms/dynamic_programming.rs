//! Small dynamic programming examples. Compare the recurrence with the code:
//! insertion, deletion, and substitution each cost one.

/// Levenshtein distance measured in Unicode scalar values (`char`), not bytes
/// or user-perceived grapheme clusters. O(mn) time and O(min(m, n)) space.
#[must_use]
pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (long, short) = if a.len() >= b.len() {
        (&a, &b)
    } else {
        (&b, &a)
    };
    let mut previous: Vec<usize> = (0..=short.len()).collect();
    let mut current = vec![0; short.len() + 1];
    for (row, &left) in long.iter().enumerate() {
        current[0] = row + 1;
        for (column, &right) in short.iter().enumerate() {
            current[column + 1] = (previous[column + 1] + 1)
                .min(current[column] + 1)
                .min(previous[column] + usize::from(left != right));
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[short.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn examples_and_unicode() {
        assert_eq!(edit_distance("", "abc"), 3);
        assert_eq!(edit_distance("kitten", "sitting"), 3);
        assert_eq!(edit_distance("é", "e"), 1);
        assert_eq!(edit_distance("rust", "rust"), 0);
        assert_eq!(edit_distance("abc", ""), 3);
    }
}
