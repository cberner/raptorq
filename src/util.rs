pub fn get_both_indices<T>(vector: &mut Vec<T>, i: usize, j: usize) -> (&mut T, &mut T) {
    assert_ne!(i, j);
    if i < j {
        let (first, last) = vector.split_at_mut(j);
        return (&mut first[i], &mut last[0])
    }
    else {
        let (first, last) = vector.split_at_mut(i);
        return (&mut last[0], &mut first[j])
    }
}