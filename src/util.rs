// Get two non-overlapping ranges starting at i & j, both with length len
pub fn get_both_ranges<T>(
    vector: &mut [T],
    i: usize,
    j: usize,
    len: usize,
) -> (&mut [T], &mut [T]) {
    debug_assert_ne!(i, j);
    debug_assert!(i + len <= vector.len());
    debug_assert!(j + len <= vector.len());
    if i < j {
        debug_assert!(i + len <= j);
        let (first, last) = vector.split_at_mut(j);
        return (&mut first[i..(i + len)], &mut last[0..len]);
    } else {
        debug_assert!(j + len <= i);
        let (first, last) = vector.split_at_mut(i);
        return (&mut last[0..len], &mut first[j..(j + len)]);
    }
}

pub fn get_both_indices<T>(vector: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    debug_assert_ne!(i, j);
    debug_assert!(i < vector.len());
    debug_assert!(j < vector.len());
    if i < j {
        let (first, last) = vector.split_at_mut(j);
        return (&mut first[i], &mut last[0]);
    } else {
        let (first, last) = vector.split_at_mut(i);
        return (&mut last[0], &mut first[j]);
    }
}
